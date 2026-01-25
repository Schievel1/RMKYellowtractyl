use rmk::event::{LayerChangeEvent, publish_controller_event};
use rmk_macro::controller;
use rmk::event::PointingSetCpiEvent;

#[controller(subscribe = [LayerChangeEvent])]
pub struct PointingDeviceController {
    current_layer: u8,
}

impl PointingDeviceController {
    pub fn new() -> Self {
        Self {
            current_layer: 0,
        }
    }

    async fn on_layer_change_event(&mut self, event: LayerChangeEvent) {
        if event.layer != self.current_layer {
            self.current_layer = event.layer;
        }

        match event.layer {
            0 => {
                publish_controller_event(PointingSetCpiEvent {
                    device_id: 0,
                    cpi: 1600,
                });
            }
            1 => {
                publish_controller_event(PointingSetCpiEvent {
                    device_id: 0,
                    cpi: 200,
                });
            }
            _ => {}
        }
    }
}

