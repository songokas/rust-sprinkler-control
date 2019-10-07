use std::collections::{HashMap};
use chrono::{Local, DateTime};

use arduino_mqtt_pin::pin::PinOperation;

#[derive(Debug, Clone)]
pub struct State
{
    pins: HashMap<u8, PinOperation>
}

impl State
{
    pub fn new() -> State
    {
        State { pins: HashMap::new()}
    }

    pub fn update(&self, op: &PinOperation) -> State
    {
        let mut hash = self.pins.clone();
        let new_op = op.clone();
        hash.insert(op.pin_state.pin, new_op);
        State { pins: hash }
    }

    pub fn is_on(&self, pin: u8) -> bool
    {
        self.pins.get(&pin).map(|op| op.pin_state.value.is_on()).unwrap_or(false)
    }

    pub fn count(&self) -> usize
    {
        self.pins.iter().fold(0, |count, (_, op)| if op.pin_state.value.is_on() { return count + 1 } else { count })
    }

    pub fn should_be_manual_until(&self, pin: u8, dt: &DateTime<Local>) -> bool
    {
        self.pins.get(&pin).and_then(| op | op.pin_state.until).map(|op_dt| &op_dt > dt).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests
{
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use arduino_mqtt_pin::pin::{PinState, PinValue};

    #[test]
    fn test_should_be_manual()
    {
        let mut state = State::new();
        let dt = Local::now();
        let expected_dt = Local::now() - chrono::Duration::minutes(1);
        let future_dt = Local::now() + chrono::Duration::minutes(1);
        state = state.update(&PinOperation { pin_state: PinState {pin: 3, value: PinValue::Digital(true), dt: Local::now(), until: Some(dt) }, node: "node".to_string() });
        assert_eq!(state.should_be_manual_until(3, &expected_dt), true);
        assert_eq!(state.should_be_manual_until(2, &expected_dt), false);
        assert_eq!(state.should_be_manual_until(3, &future_dt), false);
    }
}
