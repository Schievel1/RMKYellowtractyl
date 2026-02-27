use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use defmt::info;
use rmk::channel::KEYBOARD_REPORT_CHANNEL;
use rmk::event::publish_event;
use rmk::event::KeyboardEvent;
use rmk::event::LayerChangeEvent;
use rmk::hid::Report;
use rmk::keymap::KeyMap;
use rmk::types::action::Action;
use rmk::types::action::KeyAction;
use rmk_macro::processor;
use rmk_macro::event;
use usbd_hid::descriptor::MouseReport;

static JIGGLE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[event(channel_size = 2)]
#[derive(Clone, Copy, Debug)]
pub struct JiggleEvent(pub bool);

#[processor(subscribe = [LayerChangeEvent, KeyboardEvent], poll_interval = 1000)]
pub struct JiggleController<
    'a,
    const ROW: usize,
    const COL: usize,
    const NUM_LAYER: usize,
    const NUM_ENCODER: usize,
> {
    tick_tock: bool,
    current_layer: u8,
    keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>,
}

impl<'a, const ROW: usize, const COL: usize, const NUM_LAYER: usize, const NUM_ENCODER: usize>
    JiggleController<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>
{
    pub fn new(keymap: &'a RefCell<KeyMap<'a, ROW, COL, NUM_LAYER, NUM_ENCODER>>) -> Self {
        Self {
            tick_tock: false,
            current_layer: 0,
            keymap,
        }
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        if event.layer != self.current_layer {
            self.current_layer = event.layer;
        }
    }

    pub async fn on_keyboard_event(&mut self, event: KeyboardEvent) {
        let keyevent = self
            .keymap
            .borrow()
            .get_action_at(event.pos, self.current_layer as usize);
        if event.pressed {
            match keyevent {
                KeyAction::Single(Action::User(0)) => {
                    info!("Got KeyCode::User0");
                    let current = JIGGLE_ACTIVE.load(Ordering::SeqCst);
                    info!("Jiggle was {}, storing {}", current, !current);
                    JIGGLE_ACTIVE.store(!current, Ordering::SeqCst);
                    publish_event(JiggleEvent(!current));
                }
                _ => {}
            }
        }
    }

    pub async fn poll(&mut self) {
        if JIGGLE_ACTIVE.load(Ordering::SeqCst) {
            let mouse_report = MouseReport {
                buttons: 0,
                x: if self.tick_tock { 20 } else { -20 },
                y: if self.tick_tock { 20 } else { -20 },
                wheel: 0,
                pan: 0,
            };
            info!("Jiggle Jiggle");
            KEYBOARD_REPORT_CHANNEL
                .send(Report::MouseReport(mouse_report))
                .await;
            self.tick_tock = !self.tick_tock;
        }
    }
}
