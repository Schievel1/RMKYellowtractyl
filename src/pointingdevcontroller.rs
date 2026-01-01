use rmk::channel::{CONTROLLER_CHANNEL, ControllerSub, ControllerPub, send_controller_event};
use rmk::controller::Controller;
use rmk::event::{ControllerEvent, PointingEvent};
use rmk::event::PointingEvent::PointingSetCpi;

// Debug
use embassy_time::Timer;

pub struct PointingDeviceController {
    sub: ControllerSub,
    publ: ControllerPub,
    current_layer: u8,
}

impl PointingDeviceController {
    pub fn new() -> Self {
        Self {
            sub: CONTROLLER_CHANNEL.subscriber().unwrap(),
            publ: CONTROLLER_CHANNEL.publisher().unwrap(),
            current_layer: 0,
        }
    }

    async fn handle_layer_change(&mut self, new_layer: u8) {
        if new_layer != self.current_layer {
            self.current_layer = new_layer;

            // Emit pointing device commands based on layer
            match new_layer {
                0 => {
                    send_controller_event(&mut self.publ, ControllerEvent::PointingContEvent((0, PointingSetCpi(1600))));
                }
                1 => {
                    send_controller_event(&mut self.publ, ControllerEvent::PointingContEvent((0, PointingSetCpi(200))));
                }
                _ => {}
            }
        }
    }

}

impl Controller for PointingDeviceController {
    type Event = ControllerEvent;

    async fn process_event(&mut self, event: Self::Event) {
        match event {
            ControllerEvent::Layer(layer) => {
                self.handle_layer_change(layer).await;
            }
            _ => {}
        }
    }

    async fn next_message(&mut self) -> Self::Event {
        self.sub.next_message_pure().await
    }

}

/// Debug function that emits pointing device events with 1s delays
pub async fn debug_pointing_device_events(mut publ: ControllerPub) {
    loop {
        // Send CPI change to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointingSetCpi(800))));
        Timer::after_secs(1).await;

        // Send poll interval change to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointingSetPollIntervall(4000))));
        Timer::after_secs(1).await;

        // Send rotational transform angle to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointingSetRotTransAngle(-15))));
        Timer::after_secs(1).await;

        // Send liftoff distance to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointigSetLiftoffDist(8))));
        Timer::after_secs(1).await;

        // Send force awake mode to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointingSetForceAwake(true))));
        Timer::after_secs(1).await;

        // Send invert X to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointingSetInvertX(true))));
        Timer::after_secs(1).await;

        // Send invert Y to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointingSetInvertY(false))));
        Timer::after_secs(1).await;

        // Send swap X/Y to device 0
        send_controller_event(&mut publ,ControllerEvent::PointingContEvent((0, PointingEvent::PointingSwapXY(false))));
        Timer::after_secs(1).await;
    }
}
