use std::vec::Vec;
use yaml_rust::{Yaml};
use chrono::{Local};
use mosquitto_client::{Mosquitto};

use log::{debug, warn};

use crate::zone::Zone;
use crate::state::State;

pub fn send(client: &Mosquitto, node: &str, pin: u8, value: u16) -> Result<i32, mosquitto_client::Error>
{
    client.publish(&format!("{node}/set/json", node=node), format!("{{\"pin\": {id}, \"set\": {value}}}", id=pin, value=value).as_bytes(), 0, true)
}

pub fn create_zones(yaml: &Yaml) -> Result<Vec<Zone>, &str>
{
    let mut v = Vec::new();
    let zones = yaml["zones"].as_hash();//.ok_or("Failed to read zones")?;
    if !zones.is_some() {
       return Err("Failed to parse zones")
    }
    for (key, value) in zones.unwrap() {
        if !key.as_str().is_some() {
            continue;
        }
        Zone::from_yaml(key.as_str().unwrap(), value)
            .map(|zone| v.push(zone))
            .ok_or("Failed to parse zone yaml")?;
    }
    return Ok(v);
}

pub fn apply_states(client: &Mosquitto, remote_state: &State, local_state: &State, node: &str, zones: &Vec<Zone>) -> u8
{
    let now = Local::now();
    let now_time = now.time();
    let mut count = 0;
    for zone in zones {
        debug!("Zone name: {} pin: {}, local: {} remote: {}", zone.name, zone.control_pin, local_state.is_on(zone.control_pin), remote_state.is_on(zone.control_pin));
        let mut apply_value = None;
        if local_state.should_be_manual_until(zone.control_pin, &now) {
            if local_state.is_on(zone.control_pin) && !remote_state.is_on(zone.control_pin) {
                apply_value = Some(1);
            } else if !local_state.is_on(zone.control_pin) && remote_state.is_on(zone.control_pin) {
                apply_value = Some(0);
            }
        } else if !remote_state.is_on(zone.control_pin) && zone.should_be_on(now_time) {
            apply_value = Some(1);
        } else if remote_state.is_on(zone.control_pin) && !zone.should_be_on(now_time) {
            apply_value = Some(0);
        }

        if apply_value.is_some() {
            let result = send(client, node, zone.control_pin, apply_value.unwrap());
            if result.is_ok() {
                debug!("Sent to {}: pin {} value {}", node, zone.control_pin, apply_value.unwrap());
                count+=1;
            } else {
                warn!("Failed to update pin: {}", zone.control_pin);
            }
        }

    }
    return count;
}

