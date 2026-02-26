use crate::jigglemode::JiggleEvent;
use core::fmt::Write;
use defmt::debug;
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_6X12, FONT_6X9, FONT_7X13},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use rmk::event::LedIndicatorEvent;
use rmk::event::WpmUpdateEvent;
use rmk::heapless::String;
use rmk::{event::LayerChangeEvent, types::led_indicator::LedIndicator};
use rmk_macro::processor;
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::size::DisplaySizeAsync;
use ssd1306::Ssd1306Async;

const FONT: MonoFont<'_> = FONT_6X10;

#[processor(subscribe = [LayerChangeEvent, LedIndicatorEvent, JiggleEvent, WpmUpdateEvent], poll_interval = 50)]
pub struct Ssd1306Controller<'a, DI, SIZE>
where
    SIZE: DisplaySizeAsync,
    DI: AsyncWriteOnlyDataCommand,
{
    current_indicators: LedIndicator,
    current_layer: u8,
    jiggle_active: bool,
    current_wpm: u16,
    text_style_norm: MonoTextStyle<'a, BinaryColor>,
    text_style_inv: MonoTextStyle<'a, BinaryColor>,
    char_width: u32,
    char_height: u32,
    display: Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>,
}

impl<'a, DI, SIZE> Ssd1306Controller<'a, DI, SIZE>
where
    SIZE: DisplaySizeAsync,
    DI: AsyncWriteOnlyDataCommand,
{
    pub fn new(display: Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>) -> Self {
        let text_style_norm = MonoTextStyleBuilder::new()
            .font(&FONT)
            .text_color(BinaryColor::On)
            .build();

        let text_style_inv = MonoTextStyleBuilder::new()
            .font(&FONT)
            .text_color(BinaryColor::Off)
            .background_color(BinaryColor::On)
            .build();
        let char_width = FONT.character_size.width;
        let char_height = FONT.character_size.height;

        Self {
            current_indicators: 0.into(),
            current_layer: 0,
            current_wpm: 0,
            jiggle_active: false,
            text_style_norm,
            text_style_inv,
            char_width,
            char_height,
            display,
        }
    }

    async fn on_wpm_update_event(&mut self, event: WpmUpdateEvent) {
        self.current_wpm = event.wpm;
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        if event.layer != self.current_layer {
            self.current_layer = event.layer;
        }
    }

    async fn on_led_indicator_event(&mut self, event: LedIndicatorEvent) {
        debug!("got led ind event");
        self.current_indicators = event.indicator;
    }

    async fn on_jiggle_event(&mut self, event: JiggleEvent) {
        debug!("got jiggle event: {}", event.0);
        self.jiggle_active = event.0;
    }

    fn draw_indicators(&mut self, y: i32) {
        let indicators = [
            (
                "N",
                if self.current_indicators.num_lock() {
                    self.text_style_inv
                } else {
                    self.text_style_norm
                },
            ),
            (
                "C",
                if self.current_indicators.caps_lock() {
                    self.text_style_inv
                } else {
                    self.text_style_norm
                },
            ),
            (
                "S",
                if self.current_indicators.scroll_lock() {
                    self.text_style_inv
                } else {
                    self.text_style_norm
                },
            ),
        ];

        let mut cursor_x = 0;

        for (i, (ch, style)) in indicators.iter().enumerate() {
            Text::with_baseline(ch, Point::new(cursor_x, y), *style, Baseline::Top)
                .draw(&mut self.display)
                .unwrap();

            cursor_x += self.char_width as i32;

            if i < indicators.len() - 1 {
                Text::with_baseline(
                    " ",
                    Point::new(cursor_x, y),
                    self.text_style_norm,
                    Baseline::Top,
                )
                .draw(&mut self.display)
                .unwrap();

                cursor_x += self.char_width as i32;
            }
        }
    }
    fn draw_layer(&mut self, y: i32) {
        let layer_name = match self.current_layer {
            0 => "DVORA",
            1 => "LOWER",
            2 => "RAISE",
            _ => "UNKNO",
        };

        Text::with_baseline(
            layer_name,
            Point::new(0, y),
            self.text_style_norm,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .unwrap();
    }
    fn draw_compose_jiggle(&mut self, y: i32) {
        let compose_parts = [
            ("CO", self.current_indicators.compose()),
            ("JI", self.jiggle_active),
        ];

        let mut cursor_x = 0;

        for (i, (part, invert)) in compose_parts.iter().enumerate() {
            let style = if *invert {
                self.text_style_inv
            } else {
                self.text_style_norm
            };

            Text::with_baseline(part, Point::new(cursor_x, y), style, Baseline::Top)
                .draw(&mut self.display)
                .unwrap();

            cursor_x += self.char_width as i32 * part.len() as i32;

            if i < compose_parts.len() - 1 {
                Text::with_baseline(
                    " ",
                    Point::new(cursor_x, y),
                    self.text_style_norm,
                    Baseline::Top,
                )
                .draw(&mut self.display)
                .unwrap();

                cursor_x += self.char_width as i32;
            }
        }
    }
    fn draw_wpm(&mut self, y: i32) {
        let mut wpm_text: String<16> = String::new();
        write!(wpm_text, "W:{:>3}", self.current_wpm).unwrap();

        Text::with_baseline(
            &wpm_text,
            Point::new(0, y),
            self.text_style_norm,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .unwrap();
    }

    fn draw_cat(&mut self, position: Point) {
        // Rust-kompatibles Array
        // let mut cat_idle_1_rust = [0; 32*(40/8)];
        // qmkpages_to_emedded_graphics_lines::<32, 40>(&CAT_IDLE_1, &mut cat_idle_1_rust);

        let image = ImageRaw::<BinaryColor>::new(&CAT_IDLE_EG, 32);

        Image::new(&image, position)
            .draw(&mut self.display)
            .unwrap();
    }
    fn draw_test(&mut self, pos: Point) {
        let mut buf = [0u8; 32*(40/8)];
        qmkpages_to_emedded_graphics_lines::<32, 40>(&TEST_PATTERN, &mut buf);
        let raw = ImageRaw::<BinaryColor>::new(&buf, 32);
        Image::new(&raw, pos).draw(&mut self.display).unwrap();
    }

    pub async fn poll(&mut self) {
        self.display.clear_buffer();

        let line_height = self.char_height as i32 + 2;
        let mut y = 0;

        self.draw_indicators(y);
        y += line_height;

        self.draw_layer(y);
        y += line_height;

        self.draw_compose_jiggle(y);
        y += line_height;

        self.draw_wpm(y);

        self.draw_cat(Point::new(0, 85));
        // self.draw_test(Point::new(0, 60));

        self.display.flush().await.unwrap();
    }
}
const TEST_PATTERN: [u8; 160] = [
    0x02,0x0A,0xFF,0xFF,0xA0,0x0A,0xA0,0x0A,0x01,0x00,0x01,0x01,0x01,0x00,0x00,0x01,0x01,0x00,0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01,
    0x00,0xFF,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
    0x01,0x0A,0xAA,0x00,0xA0,0x0A,0xA0,0x0A,0x01,0x00,0x01,0x01,0x01,0x00,0x00,0x01,0x01,0x00,0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x01,
    0x00,0x00,0x00,0xAA,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
    0x01,0x00,0x00,0x00,0xAA,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,

];
const CAT_IDLE_1: [u8; 160] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xc0, 0x60, 0x10, 0x10, 0x60, 0xc0, 0x00, 0x80, 0x40, 0x40, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x40, 0x40, 0x80, 0x00, 0xc0, 0x60, 0x10, 0x10, 0x60, 0xc0, 0x00, 0x00,
    0x80, 0x70, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x60, 0x60, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x60, 0x60, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x0f, 0x78, 0xc0,
    0x1f, 0x10, 0x10, 0x10, 0x10, 0x70, 0xc0, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x71, 0x12, 0x12, 0x11, 0x12, 0x72, 0xc1, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x70, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x02, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const CAT_IDLE_EG: [u8; 160] = {
    let mut buf = [0u8; 32*(40/8)];
    qmkpages_to_emedded_graphics_lines::<32, 40>(&CAT_IDLE_1, &mut buf);
    buf
};

const fn qmkpages_to_emedded_graphics_lines<const W: usize, const H: usize>(
    input: &[u8],
    output: &mut [u8],
) {
    const fn const_get_bit(b: u8, n: usize) -> u8 {
        ((b >> n) & 1) as u8
    }
    if W % 8 != 0 {
        panic!("width must be multiple of 8");
    }
    if output.len() != (W * H + 7) / 8 {
        panic!("output length must be width*(height/8)");
    }
    let bytes_per_row = W / 8;
    let mut row = 0;
    while row < H {
        let mut byte_in_row = 0;
        while byte_in_row < bytes_per_row {
            let mut bit_in_byte = 0;
            while bit_in_byte < 8 {
                // QMK Page-Mode: vertikale Spalten; row/8 wählt die Page
                let src_byte = input[bit_in_byte + byte_in_row * 8 + (row / 8) * W];
                let bit = const_get_bit(src_byte, row % 8);
                // Zeilenmajor-Index im Ausgabepuffer
                output[byte_in_row + row * bytes_per_row] |= bit << (7 - bit_in_byte);
                bit_in_byte += 1;
            }
            byte_in_row += 1;
        }
        row += 1;
    }
}
