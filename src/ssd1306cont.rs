use defmt::debug;
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics::mono_font::MonoTextStyle;
use rmk::event::LayerChangeEvent;
use rmk_macro::processor;
use ssd1306::mode::BufferedGraphicsModeAsync;
use ssd1306::size::DisplaySizeAsync;

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_6X12, FONT_6X9, FONT_7X13},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::Ssd1306Async;

#[processor(subscribe = [LayerChangeEvent], poll_interval = 50)]
pub struct Ssd1306Controller<'a, DI, SIZE>
where
    SIZE: DisplaySizeAsync,
    DI: AsyncWriteOnlyDataCommand,
{
    current_modifiers: u8,
    current_layer: u8,
    text_style: MonoTextStyle<'a, BinaryColor>,
    display: Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>,
}

impl<'a, DI, SIZE> Ssd1306Controller<'a, DI, SIZE>
where
    SIZE: DisplaySizeAsync,
    DI: AsyncWriteOnlyDataCommand,
{
    pub fn new(display: Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>) -> Self {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Self {
            current_modifiers: 0,
            current_layer: 0,
            text_style,
            display,
        }
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        if event.layer != self.current_layer {
            self.current_layer = event.layer;
        }
    }

    pub async fn poll(&mut self) {
        self.display.clear_buffer();
        Text::with_baseline(
            "N C S",
            Point::new(0, 0),
            self.text_style,
            Baseline::Top,
        )
        .draw(&mut self.display)
        .unwrap();
        self.display.flush().await.unwrap();
    }
}
