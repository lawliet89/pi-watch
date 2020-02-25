use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use tokio::fs;

#[derive(snafu::Snafu, Debug)]
pub enum Error {
    #[snafu(display("IO Error while {}: {}", context, source))]
    IOError {
        source: std::io::Error,
        context: String,
    },
    #[snafu(display("Error parsing value as float: {}", source))]
    ParseFloatError { source: std::num::ParseFloatError },
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        if let Self::IOError { source, .. } = self {
            return source.kind() == std::io::ErrorKind::NotFound
        }
        false
    }
}

static TEMPERATURE_SENSORS_GLOB: &[&str] = &[
    "/sys/class/hwmon/hwmon*/temp*_*",
    "/sys/class/hwmon/hwmon*/device/temp*_*'",
    "/sys/devices/platform/coretemp.*/hwmon/hwmon*/temp*_*",
];

pub struct Stat {
    hwmon_temperature: HashSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HwmonTemperature {
    pub name: String,
    pub value: f32,

    pub high: Option<f32>,
    pub critical: Option<f32>,
    pub label: Option<String>,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            hwmon_temperature: Self::hwmon_temperature(),
        }
    }

    pub async fn temperature(&self) -> Vec<Result<HwmonTemperature, Error>> {
        futures::future::join_all(
            self.hwmon_temperature
                .iter()
                .map(|base| Self::read_hwmon_temperature(&base)),
        )
        .await
    }

    pub async fn read_hwmon_temperature(base: &str) -> Result<HwmonTemperature, Error> {
        let name = read_and_trim(
            Path::new(base)
                .parent()
                .expect("not to be root")
                .join("name"),
        )
        .await?;
        let value = read_as_float(format!("{}_input", base)).await? / 1000.;

        let high = read_as_float(format!("{}_max", base))
            .await
            .map(|f| f / 1000.)
            .ok();
        let critical = read_as_float(format!("{}_crit", base))
            .await
            .map(|f| f / 1000.)
            .ok();
        let label = read_and_trim(format!("{}_label", base)).await.ok();
        Ok(HwmonTemperature {
            name,
            value,
            high,
            critical,
            label,
        })
    }

    // From https://github.com/giampaolo/psutil/blob/544e9daa4f66a9f80d7bf6c7886d693ee42f0a13/psutil/_pslinux.py#L1190
    fn hwmon_temperature() -> HashSet<String> {
        TEMPERATURE_SENSORS_GLOB
            .iter()
            .map(|path| glob::glob(path).expect("Glob pattern to not error"))
            .flatten()
            .filter_map(|path| match path {
                Ok(s) => Some(s),
                Err(e) => {
                    log::warn!("Ignoring due to error: {}", e);
                    None
                }
            })
            .filter_map(|p| match p.to_str() {
                Some(p) => p.split('_').next().map(|s| s.to_string()),
                None => {
                    log::warn!("Ignoring path {:#?} because it is not unicode", p);
                    None
                }
            })
            .collect()
    }
}

async fn read_and_trim(path: impl AsRef<Path>) -> Result<String, Error> {
    let path = path.as_ref();
    let mut content = fs::read_to_string(&path).await.context(IOError {
        context: format!("reading value from {:#?}", path),
    })?;
    if content.ends_with('\n') {
        content.pop();
    }
    if content.ends_with('\r') {
        content.pop();
    }
    Ok(content)
}

async fn read_as_float(path: impl AsRef<Path>) -> Result<f32, Error> {
    let value = read_and_trim(path).await?;
    value.parse().context(ParseFloatError {})
}
