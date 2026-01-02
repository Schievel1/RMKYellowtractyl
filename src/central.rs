#![no_main]
#![no_std]

#[macro_use]
mod keymap;
#[macro_use]
mod macros;
mod vial;

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::flash::{Async, Flash};
use embassy_rp::gpio::Input;
use embassy_rp::peripherals::{UART0, USB};
use embassy_rp::uart::{self, BufferedUart};
use embassy_rp::usb::{Driver, InterruptHandler};
use rmk::channel::EVENT_CHANNEL;
use rmk::config::{
    BehaviorConfig, DeviceConfig, MorsesConfig, PositionalConfig, RmkConfig, StorageConfig,
    VialConfig,
};
use rmk::heapless::Vec;
use rmk::types::action::{MorseMode, MorseProfile};
// use rmk::config::macro_config::KeyboardMacrosConfig;
// use rmk::config::CombosConfig;
// use rmk::config::TapConfig;
use rmk::controller::EventController;
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::input_device::Runnable;
use rmk::join_all;
use rmk::keyboard::Keyboard;
use rmk::matrix::{Matrix, OffsetMatrixWrapper};
use rmk::split::central::run_peripheral_manager;
use rmk::split::SPLIT_MESSAGE_MAX_SIZE;
use rmk::{initialize_keymap_and_storage, run_devices, run_processor_chain, run_rmk};
use static_cell::StaticCell;
use vial::{VIAL_KEYBOARD_DEF, VIAL_KEYBOARD_ID};
use {defmt_rtt as _, panic_probe as _};
pub mod pmw3360srom;

pub mod pointingdevcontroller;
use crate::pointingdevcontroller::PointingDeviceController;
// Debug
use crate::pointingdevcontroller::debug_pointing_device_events;
use rmk::channel::CONTROLLER_CHANNEL;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    UART0_IRQ => uart::BufferedInterruptHandler<UART0>;
});

const FLASH_SIZE: usize = 2 * 1024 * 1024;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK start!");
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Create the usb driver, from the HAL
    let driver = Driver::new(p.USB, Irqs);

    // Pin config
    let (row_pins, col_pins) = config_matrix_pins_rp!(
        peripherals: p,
        input: [PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9],
        output: [PIN_10, PIN_11, PIN_12, PIN_13, PIN_14, PIN_15]);

    // Use internal flash to emulate eeprom
    // Both blocking and async flash are support, use different API
    // let flash = Flash::<_, Blocking, FLASH_SIZE>::new_blocking(p.FLASH);
    let flash = Flash::<_, Async, FLASH_SIZE>::new(p.FLASH, p.DMA_CH0);

    let keyboard_device_config = DeviceConfig {
        vid: 0x44dd,
        pid: 0x3536,
        manufacturer: "Schievel (Pascal Jaeger)",
        product_name: "YellowTractyl",
        serial_number: "vial:f64c2b3c:000001",
    };

    let vial_config = VialConfig::new(VIAL_KEYBOARD_ID, VIAL_KEYBOARD_DEF, &[(0, 0), (2, 0)]); // Keys at (row=0,col=0) and (row=2,col=0) (~ and ESC)

    let rmk_config = RmkConfig {
        device_config: keyboard_device_config,
        vial_config,
        ..Default::default()
    };

    static TX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    static RX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    let uart_receiver = BufferedUart::new(
        p.UART0,
        p.PIN_0,
        p.PIN_1,
        Irqs,
        tx_buf,
        rx_buf,
        uart::Config::default(),
    );
    use embassy_time::Duration;

    // Initialize the storage and keymap
    let mut default_keymap = keymap::get_default_keymap();
    let mut behavior_config = BehaviorConfig {
        keyboard_macros: keymap::get_macros(),
        combo: keymap::get_combos(),
        morse: MorsesConfig {
            enable_flow_tap: false,
            prior_idle_time: Duration::from_millis(250u64),
            default_profile: MorseProfile::new(
                Some(false),
                Some(MorseMode::PermissiveHold),
                Some(150u16),
                Some(250u16),
            ),
            morses: Vec::new(),
        },
        ..Default::default()
    };
    // let mut behavior_config = BehaviorConfig::default();
    let storage_config = StorageConfig::default();
    // let storage_config = StorageConfig {
    // clear_storage: true,
    // clear_layout: true,
    // ..Default::default()
    // };
    let mut per_key_config = PositionalConfig::default();
    let (keymap, mut storage) = initialize_keymap_and_storage(
        &mut default_keymap,
        flash,
        &storage_config,
        &mut behavior_config,
        &mut per_key_config,
    )
    .await;

    // Initialize the matrix + keyboard
    let debouncer = DefaultDebouncer::new();
    let mut matrix = OffsetMatrixWrapper::<_, _, _, 0, 6>(Matrix::<_, _, _, 6, 6, true>::new(
        row_pins, col_pins, debouncer,
    ));
    let mut keyboard = Keyboard::new(&keymap);

    // PMW sensor
    use embassy_embedded_hal::adapter::BlockingAsync;
    use embassy_rp::gpio::Output;
    use embassy_rp::gpio::{Level, Pull};
    use embassy_rp::spi::{Config, Phase, Polarity, Spi};
    use rmk::input_device::pmw33xx::{Pmw3360Spec, Pmw33xx, Pmw33xxConfig};
    use rmk::input_device::pointing::PointingDevice;

    let mut spi_cfg = Config::default();
    // // MODE_3 = Polarity::IdleHigh + Phase::CaptureOnSecondTransition
    spi_cfg.polarity = Polarity::IdleHigh;
    spi_cfg.phase = Phase::CaptureOnSecondTransition;
    spi_cfg.frequency = 2_000_000;

    // // Create GPIO pins
    let pmw3360_sck = p.PIN_18;
    let pmw3360_mosi = p.PIN_19;
    let pmw3360_miso = p.PIN_16;
    let pmw3360_cs = Output::new(p.PIN_17, Level::High);
    let pmw3360_irq = Input::new(p.PIN_20, Pull::Up);

    // Create the SPI bus
    // let pmw3360_spi = Spi::new(p.SPI0, pmw3360_sck,pmw3360_mosi,pmw3360_miso, p.DMA_CH2, p.DMA_CH3, spi_cfg);
    let pmw3360_spi = Spi::new_blocking(p.SPI0, pmw3360_sck, pmw3360_mosi, pmw3360_miso, spi_cfg);
    let pmw3360_spi = BlockingAsync::new(pmw3360_spi);

    // Initialize PMW3360 mouse sensor
    let pmw3360_config = Pmw33xxConfig {
        res_cpi: 1600,
        rot_trans_angle: -15,
        liftoff_dist: 0x08,
        swap_xy: false,
        invert_x: true,
        invert_y: false,
        ..Default::default()
    };

    // Create the sensor device
    let mut pmw3360_device = PointingDevice::<Pmw33xx<_, _, _, Pmw3360Spec>>::new_with_firmware(
        0,
        pmw3360_spi,
        pmw3360_cs,
        Some(pmw3360_irq),
        pmw3360_config,
        crate::pmw3360srom::PMW3360_SROM,
    );

    use rmk::input_device::pointing::PointingProcessor;

    let mut pmw3360_processor = PointingProcessor::new(&keymap);

    // Initialize pointing device controller
    // this is for detecting layer changes and sending controller events to the PMW3360
    let mut pointing_controller = PointingDeviceController::new();

    // Debug
    // let cont_pub = CONTROLLER_CHANNEL.publisher().unwrap();

    join_all!(
        run_devices! (
            (matrix, pmw3360_device) => EVENT_CHANNEL,
        ),
        run_processor_chain! {
            EVENT_CHANNEL => [pmw3360_processor],
        },
        keyboard.run(),
        pointing_controller.event_loop(),
        run_peripheral_manager::<6, 6, 0, 0, _>(0, uart_receiver),
        run_rmk(&keymap, driver, &mut storage, rmk_config) // ,debug_pointing_device_events(cont_pub)
    )
    .await;
}
