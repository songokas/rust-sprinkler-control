#![allow(unused_variables)]

#[macro_use]
extern crate clap;

use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::thread;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use yaml_rust::{YamlLoader};
use mosquitto_client::{Mosquitto};
use clap::{App};
use env_logger::Env;

use log::{debug, warn, error, info};

pub mod config;
pub mod state;
pub mod helper;
pub mod zone;

use crate::helper::{apply_states, create_zones};
use crate::state::State;
use crate::config::Config;

use arduino_mqtt_pin::pin::PinOperation;

fn main() -> Result<(), Error>
{
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let config = matches.value_of("config").unwrap_or("config.conf");
    let verbosity = matches.occurrences_of("verbose");

    env_logger::from_env(Env::default().default_filter_or(match verbosity { 1 => "debug", 2 => "trace", _ => "info"})).init();

    info!("Using config: {}", config);

    let mut yaml_file = File::open(config)?;
    let mut contents = String::new();
    yaml_file.read_to_string(&mut contents)?;

    debug!("Config loaded: {} Verbosity: {}", config, verbosity);
    //let d: String = serde_yaml::from_reader(f)?;

    let yaml_config = YamlLoader::load_from_str(&contents)
        .map_err(|err| error!("{:?}", err))
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Unable to parse yaml file"))?;

    //println!("Loaded config: {:?}", yaml_config[0]);

    let config = Config::from_yaml(&yaml_config[0])
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Unable to parse config"))?;

    let zones = create_zones(&yaml_config[0])
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Unable to parse zones"))?;

    let local_state = RefCell::new(State::new());
    let remote_state = RefCell::new(State::new());

    let client = Mosquitto::new(&config.name);
    client.connect(&config.host, 1883)
        .map_err(|_| Error::new(ErrorKind::NotConnected, format!("Unable to connect to host: {}", config.host)))?;

    /*
     * receive remote on :
     * prefix/nodes/some-node-id/analog/set/3 1
     * receive local on:
     * prefix/master/analog/timeout/3
     */
    let remote_set = format!("{}/current/#", config.node);
    let local_set = format!("{}/master/#", config.name);

    let remote_channel = client.subscribe(&remote_set, 0)
        .map(|a| { info!("Listening for {}", remote_set); a})
        .map_err(|_| Error::new(ErrorKind::NotConnected, format!("Unable to subscribe: {}", remote_set)))?;
    let local_channel = client.subscribe(&local_set, 0)
        .map(|a| { info!("Listening for {}", local_set); a })
        .map_err(|_| Error::new(ErrorKind::NotConnected, format!("Unable to subscribe: {}", local_set)))?;

    let mut mc = client.callbacks(());
    mc.on_message(|_,msg| {

        debug!("Message received: {:?}", msg);

        let message = PinOperation::from_message(&msg);
        if !message.is_ok() {
            warn!("Failed to parse message {:?}", msg);
            warn!("{}", message.err().unwrap_or("Failed to see error"));
            return;
        }

        if remote_channel.matches(&msg) {
            let rstate = remote_state.borrow().update(&message.unwrap());
            remote_state.replace(rstate);
        } else if local_channel.matches(&msg) {
            let lstate = local_state.borrow().update(&message.unwrap());
            local_state.replace(lstate);
        }
    });
    
    loop {
        let count = apply_states(&client, &remote_state.borrow(), &local_state.borrow(), &config.node, &zones);
        if count > 0 {
            info!("States expected to change: {}", count);
        }
        client.do_loop(-1)
            .map_err(|_| Error::new(ErrorKind::NotConnected, format!("Mqtt disconnected")))?;

        thread::sleep(Duration::from_millis(2000));
    }
}




