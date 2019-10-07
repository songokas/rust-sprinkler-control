use yaml_rust::Yaml;

#[derive(Debug)]
pub struct Config
{
    pub name: String,
    pub node: String,
    pub soil_dry: u16,
    pub soil_wet: u16,
    pub host: String,
}

impl Config
{
    pub fn from_yaml(yaml: &Yaml) -> Option<Config>
    {
        let name = yaml["name"].as_str()?;
        let host = yaml["host"].as_str()?;
        let node = yaml["node"].as_str()?;
        let soil_dry = yaml["soil_sensor"]["dry"].as_i64()? as u16;
        let soil_wet = yaml["soil_sensor"]["wet"].as_i64()? as u16;
        Some(Config {name: name.to_string(), node: node.to_string(), soil_dry, soil_wet, host: host.to_string()})

    }
}


