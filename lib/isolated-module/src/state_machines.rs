use std::{ops::Add, sync::LazyLock};

use tokio::sync::Mutex;

#[derive(Debug)]
enum _State {
    On,
    Off,
}

enum _Event {
    SwitchOn,
    SwitchOff,
}
static COUNT: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(0));
impl _State {
    async fn _transition(self, event: _Event) -> Self {
        let mut count = COUNT.lock().await;
        *count += 1;
        drop(count);
        match (&self, event) {
            (_State::On, _Event::SwitchOn) => {
                println!("Transitioning to the off state");
                println!("count: {}", COUNT.lock().await);
                _State::Off
            }
            (_State::Off, _Event::SwitchOff) => {
                println!("Transitioning to the On state");
                println!("count: {}", COUNT.lock().await);
                _State::On
            }
            _ => {
                println!("No transition possible staying in the current state");
                println!("count: {} | {:?} ", COUNT.lock().await, self);
                self
            }
        }
    }
}
#[tokio::test]
async fn test_main() {
    let mut state = _State::On;

    state = state._transition(_Event::SwitchOff).await;
    state = state._transition(_Event::SwitchOn).await;
    state = state._transition(_Event::SwitchOn).await;

    match state {
        _State::On => println!("State machine is in the On state"),
        _ => println!("State machine is not in the expected state"),
    }
}
