use core::sync::atomic::{AtomicBool, Ordering};
use defmt::info;
use rmk::channel::KEYBOARD_REPORT_CHANNEL;
use rmk::event::KeyEvent;
use rmk::hid::Report;
use rmk::types::action::Action;
use rmk::types::action::KeyAction;
use rmk_macro::controller;
use usbd_hid::descriptor::MouseReport;

static JIGGLE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[controller(subscribe = [KeyEvent], poll_interval = 1000)]
pub struct JiggleController {
    tick_tock: bool,
}

impl Default for JiggleController {
    fn default() -> Self {
        Self::new()
    }
}

impl JiggleController {
    pub fn new() -> Self {
        Self { tick_tock: false }
    }

    pub async fn on_key_event(&mut self, event: KeyEvent) {
        match event {
            KeyEvent {
                keyboard_event,
                key_action: KeyAction::Single(Action::User(0)),
            } if keyboard_event.pressed => {
                info!("Got KeyCode::User0");
                let current = JIGGLE_ACTIVE.load(Ordering::SeqCst);
                info!("Jiggle was {}, storing {}", current, !current);
                JIGGLE_ACTIVE.store(!current, Ordering::SeqCst);
            }
            _ => {}
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
