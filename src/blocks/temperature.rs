use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

use crossbeam_channel::Sender;
use serde_derive::Deserialize;
use uuid::Uuid;

use crate::blocks::Update;
use crate::blocks::{Block, ConfigBlock};
use crate::config::Config;
use crate::de::deserialize_duration;
use crate::errors::*;
use crate::input::{I3BarEvent, MouseButton};
use crate::scheduler::Task;
use crate::util::FormatTemplate;
use crate::widget::{I3BarWidget, Spacing, State};
use crate::widgets::button::ButtonWidget;

pub struct Temperature {
    text: ButtonWidget,
    output: String,
    collapsed: bool,
    id: String,
    update_interval: Duration,
    maximum_good: i64,
    maximum_idle: i64,
    maximum_info: i64,
    maximum_warning: i64,
    format: FormatTemplate,
    chip: Option<String>,
    inputs: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct TemperatureConfig {
    /// Update interval in seconds
    #[serde(
        default = "TemperatureConfig::default_interval",
        deserialize_with = "deserialize_duration"
    )]
    pub interval: Duration,

    /// Collapsed by default?
    #[serde(default = "TemperatureConfig::default_collapsed")]
    pub collapsed: bool,

    /// Maximum temperature, below which state is set to good
    #[serde(default = "TemperatureConfig::default_good")]
    pub good: i64,

    /// Maximum temperature, below which state is set to idle
    #[serde(default = "TemperatureConfig::default_idle")]
    pub idle: i64,

    /// Maximum temperature, below which state is set to info
    #[serde(default = "TemperatureConfig::default_info")]
    pub info: i64,

    /// Maximum temperature, below which state is set to warning
    #[serde(default = "TemperatureConfig::default_warning")]
    pub warning: i64,

    /// Format override
    #[serde(default = "TemperatureConfig::default_format")]
    pub format: String,

    /// Chip override
    #[serde(default = "TemperatureConfig::default_chip")]
    pub chip: Option<String>,

    /// Inputs whitelist
    #[serde(default = "TemperatureConfig::default_inputs")]
    pub inputs: Option<Vec<String>>,
}

impl TemperatureConfig {
    fn default_format() -> String {
        "{average}° avg, {max}° max".to_owned()
    }

    fn default_interval() -> Duration {
        Duration::from_secs(5)
    }

    fn default_collapsed() -> bool {
        true
    }

    fn default_good() -> i64 {
        20
    }

    fn default_idle() -> i64 {
        45
    }

    fn default_info() -> i64 {
        60
    }

    fn default_warning() -> i64 {
        80
    }

    fn default_chip() -> Option<String> {
        None
    }

    fn default_inputs() -> Option<Vec<String>> {
        None
    }
}

impl ConfigBlock for Temperature {
    type Config = TemperatureConfig;

    fn new(
        block_config: Self::Config,
        config: Config,
        _tx_update_request: Sender<Task>,
    ) -> Result<Self> {
        let id = Uuid::new_v4().to_simple().to_string();
        Ok(Temperature {
            update_interval: block_config.interval,
            text: ButtonWidget::new(config, &id)
                .with_icon("thermometer")
                .with_spacing(if block_config.collapsed {
                    Spacing::Hidden
                } else {
                    Spacing::Normal
                }),
            output: String::new(),
            collapsed: block_config.collapsed,
            id,
            maximum_good: block_config.good,
            maximum_idle: block_config.idle,
            maximum_info: block_config.info,
            maximum_warning: block_config.warning,
            format: FormatTemplate::from_string(&block_config.format)
                .block_error("temperature", "Invalid format specified for temperature")?,
            chip: block_config.chip,
            inputs: block_config.inputs,
        })
    }
}

type SensorsOutput = HashMap<String, HashMap<String, serde_json::Value>>;
type InputReadings = HashMap<String, f64>;

impl Block for Temperature {
    fn update(&mut self) -> Result<Option<Update>> {
        let mut args = vec!["-j"];
        if let Some(ref chip) = &self.chip {
            args.push(chip);
        }
        let output = Command::new("sensors")
            .args(&args)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_else(|e| e.to_string());

        let parsed: SensorsOutput = serde_json::from_str(&output)
            .block_error("temperature", "sensors output is invalid")?;

        let mut temperatures: Vec<i64> = Vec::new();
        for (_chip, inputs) in parsed {
            for (input_name, input_values) in inputs {
                if let Some(ref whitelist) = self.inputs {
                    if !whitelist.contains(&input_name) {
                        continue;
                    }
                }

                let values_parsed: InputReadings = match serde_json::from_value(input_values) {
                    Ok(values) => values,
                    Err(_) => continue, // probably the "Adapter" key, just ignore.
                };

                for (value_name, value) in values_parsed {
                    if !value_name.starts_with("temp") || !value_name.ends_with("input") {
                        continue;
                    }

                    if value > -101f64 && value < 151f64 {
                        temperatures.push(value as i64);
                    } else {
                        // This error is recoverable and therefore should not stop the program
                        eprintln!("Temperature ({}) outside of range ([-100, 150])", value);
                    }
                }
            }
        }

        if !temperatures.is_empty() {
            let max: i64 = *temperatures
                .iter()
                .max()
                .block_error("temperature", "failed to get max temperature")?;
            let min: i64 = *temperatures
                .iter()
                .min()
                .block_error("temperature", "failed to get min temperature")?;
            let avg: i64 = (temperatures.iter().sum::<i64>() as f64 / temperatures.len() as f64)
                .round() as i64;

            let values = map!("{average}" => avg,
                              "{min}" => min,
                              "{max}" => max);

            self.output = self.format.render_static_str(&values)?;
            if !self.collapsed {
                self.text.set_text(self.output.clone());
            }

            let state = match max {
                m if m <= self.maximum_good => State::Good,
                m if m <= self.maximum_idle => State::Idle,
                m if m <= self.maximum_info => State::Info,
                m if m <= self.maximum_warning => State::Warning,
                _ => State::Critical,
            };

            self.text.set_state(state);
        }

        Ok(Some(self.update_interval.into()))
    }

    fn view(&self) -> Vec<&dyn I3BarWidget> {
        vec![&self.text]
    }

    fn click(&mut self, e: &I3BarEvent) -> Result<()> {
        if let Some(ref name) = e.name {
            if name.as_str() == self.id && e.button == MouseButton::Left {
                self.collapsed = !self.collapsed;
                if self.collapsed {
                    self.text.set_text(String::new());
                    self.text.set_spacing(Spacing::Hidden);
                } else {
                    self.text.set_text(self.output.clone());
                    self.text.set_spacing(Spacing::Normal);
                }
            }
        }

        Ok(())
    }

    fn id(&self) -> &str {
        &self.id
    }
}
