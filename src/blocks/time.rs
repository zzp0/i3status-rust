use std::convert::TryInto;
use std::time::Duration;

use chrono::{
    offset::{Local, Utc},
    Locale,
};
use chrono_tz::Tz;
use crossbeam_channel::Sender;
use serde_derive::Deserialize;

use crate::blocks::{Block, ConfigBlock, Update};
use crate::config::Config;
use crate::de::deserialize_duration;
use crate::errors::*;
use crate::input::{I3BarEvent, MouseButton};
use crate::scheduler::Task;
use crate::subprocess::spawn_child_async;
use crate::util::pseudo_uuid;
use crate::widget::I3BarWidget;
use crate::widgets::button::ButtonWidget;

pub struct Time {
    time: ButtonWidget,
    id: String,
    update_interval: Duration,
    format: String,
    on_click: Option<String>,
    timezone: Option<Tz>,
    locale: Option<String>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct TimeConfig {
    /// Format string.<br/> See [chrono docs](https://docs.rs/chrono/0.3.0/chrono/format/strftime/index.html#specifiers) for all options.
    #[serde(default = "TimeConfig::default_format")]
    pub format: String,

    /// Update interval in seconds
    #[serde(
        default = "TimeConfig::default_interval",
        deserialize_with = "deserialize_duration"
    )]
    pub interval: Duration,

    #[serde(default = "TimeConfig::default_on_click")]
    pub on_click: Option<String>,

    #[serde(default = "TimeConfig::default_timezone")]
    pub timezone: Option<Tz>,

    #[serde(default = "TimeConfig::default_locale")]
    pub locale: Option<String>,
}

impl TimeConfig {
    fn default_format() -> String {
        "%a %d/%m %R".to_owned()
    }

    fn default_interval() -> Duration {
        Duration::from_secs(5)
    }

    fn default_on_click() -> Option<String> {
        None
    }

    fn default_timezone() -> Option<Tz> {
        None
    }

    fn default_locale() -> Option<String> {
        None
    }
}

impl ConfigBlock for Time {
    type Config = TimeConfig;

    fn new(
        block_config: Self::Config,
        config: Config,
        _tx_update_request: Sender<Task>,
    ) -> Result<Self> {
        let i = pseudo_uuid();
        Ok(Time {
            id: i.clone(),
            format: block_config.format,
            time: ButtonWidget::new(config, i.as_str())
                .with_text("")
                .with_icon("time"),
            update_interval: block_config.interval,
            on_click: block_config.on_click,
            timezone: block_config.timezone,
            locale: block_config.locale,
        })
    }
}

impl Block for Time {
    fn update(&mut self) -> Result<Option<Update>> {
        let time = match &self.locale {
            Some(l) => {
                let locale: Locale = l
                    .as_str()
                    .try_into()
                    .block_error("time", "invalid locale")?;
                match self.timezone {
                    Some(tz) => Utc::now()
                        .with_timezone(&tz)
                        .format_localized(&self.format, locale),
                    None => Local::now().format_localized(&self.format, locale),
                }
            }
            None => match self.timezone {
                Some(tz) => Utc::now().with_timezone(&tz).format(&self.format),
                None => Local::now().format(&self.format),
            },
        };
        self.time.set_text(format!("{}", time));
        Ok(Some(self.update_interval.into()))
    }

    fn click(&mut self, e: &I3BarEvent) -> Result<()> {
        if let Some(ref name) = e.name {
            if name.as_str() == self.id {
                if let MouseButton::Left = e.button {
                    if let Some(ref cmd) = self.on_click {
                        spawn_child_async("sh", &["-c", cmd])
                            .block_error("time", "could not spawn child")?;
                    }
                }
            }
        }
        Ok(())
    }

    fn view(&self) -> Vec<&dyn I3BarWidget> {
        vec![&self.time]
    }

    fn id(&self) -> &str {
        &self.id
    }
}
