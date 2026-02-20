#![no_main]
#![no_std]

#[macro_use]
mod macros;

use defmt::*;
use embassy_embedded_hal::adapter::BlockingAsync;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Output};
use embassy_rp::i2c::I2c;
use embassy_rp::peripherals::{UART0, USB};
use embassy_rp::uart::{self, BufferedUart};
use embassy_rp::usb::InterruptHandler;
use rmk::debounce::default_debouncer::DefaultDebouncer;
use rmk::join_all;
use rmk::matrix::Matrix;
use rmk::run_all;
use rmk::split::peripheral::run_rmk_split_peripheral;
use rmk::split::SPLIT_MESSAGE_MAX_SIZE;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

pub mod ssd1306cont;
use ssd1306cont::Ssd1306Controller;

// graphics
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306Async};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
    UART0_IRQ => uart::BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("RMK start!");
    // Initialize peripherals
    let p = embassy_rp::init(Default::default());

    // Pin config
    let (row_pins, col_pins) = config_matrix_pins_rp!(
        peripherals: p,
        input: [PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9],
        output: [PIN_10, PIN_11, PIN_12, PIN_13, PIN_14, PIN_15]);

    static TX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    static RX_BUF: StaticCell<[u8; SPLIT_MESSAGE_MAX_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; SPLIT_MESSAGE_MAX_SIZE])[..];
    let uart_instance = BufferedUart::new(
        p.UART0,
        p.PIN_0,
        p.PIN_1,
        Irqs,
        tx_buf,
        rx_buf,
        uart::Config::default(),
    );

    // Define the matrix
    let debouncer = DefaultDebouncer::new();
    let mut matrix = Matrix::<_, _, _, 6, 6, true, 0, 0>::new(row_pins, col_pins, debouncer);

    // display driver
    let config = embassy_rp::i2c::Config::default();
    let i2c = I2c::new_blocking(p.I2C1, p.PIN_3, p.PIN_2, config);
    let i2c = BlockingAsync::new(i2c);

    // Create display interface with default I2C address 0x3C
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306Async::new(interface, DisplaySize128x32, DisplayRotation::Rotate90)
        .into_buffered_graphics_mode();
    display.init().await.unwrap();

    let mut ssd1306cont = Ssd1306Controller::new(display);

    // Start
    join_all!(
        run_all!(matrix, ssd1306cont),
        run_rmk_split_peripheral(uart_instance)
    )
    .await;
}
