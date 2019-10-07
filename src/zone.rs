use std::vec::Vec;
use yaml_rust::{Yaml};
use chrono::{NaiveTime};

#[derive(Debug)]
struct Interval
{
    start: NaiveTime,
    end: NaiveTime
}

#[derive(Debug)]
pub struct Zone
{
    pub name: String,
    pub sensor_pin: u8,
    pub control_pin: u8,
    times: Vec<Interval>
}

impl Zone
{

    pub fn from_yaml(name: &str, yaml: &Yaml) -> Option<Zone>
    {
        let sensor_pin = yaml["sensor_pin"].as_i64()? as u8;
        let control_pin = yaml["control_pin"].as_i64()? as u8;
        let mut v = Vec::new();
        for time in yaml["times"].as_vec()? {
            let start = NaiveTime::parse_from_str(
                &format!("{}:00", time["start"].as_str()?),
                "%H:%M:%S"
            ).ok()?;
            let end = NaiveTime::parse_from_str(
                &format!("{}:00", time["end"].as_str()?),
                "%H:%M:%S"
            ).ok()?;
            v.push(Interval {start, end});
        }
        Some(Zone {name: name.to_string(), sensor_pin, control_pin, times: v})
    }

    pub fn should_be_on(&self, now: NaiveTime) -> bool
    {
        for time in &self.times {
          if now > time.start && now < time.end {
            return true;
          }
        }
        false
    }
}
