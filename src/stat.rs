use snafu::ResultExt;
use systemstat::Platform;

use crate::errors;

static TEMPERATURE_SENSORS_GLOB: &[&str] = &[
    "/sys/class/hwmon/hwmon*/temp*_*",
    "/sys/class/hwmon/hwmon*/device/temp*_*'",
    "/sys/devices/platform/coretemp.*/hwmon/hwmon*/temp*_*",
];

pub struct Stat {
    system: systemstat::System,
}

impl Stat {
    pub fn new() -> Self {
        Self {
            system: systemstat::System::new(),
        }
    }

    pub fn temperature(&self) -> Result<f32, errors::Error> {
        Ok(self.system.cpu_temp().context(errors::IOError {
            context: "reading temperature",
        })?)
    }

    // From https://github.com/giampaolo/psutil/blob/544e9daa4f66a9f80d7bf6c7886d693ee42f0a13/psutil/_pslinux.py#L1190
    pub fn sensors_temperature() -> Vec<String> {
        TEMPERATURE_SENSORS_GLOB
            .iter()
            .map(|path| glob::glob(path).expect("Glob pattern to not error"))
            .flatten()
            .filter_map(|path| {
                match path {
                    Ok(s) => Some(s),
                    Err(e) => {
                        log::warn!("Ignoring due to error: {}", e);
                        None
                    }
                }
            })
            .filter_map(|p| {
                match p.to_str() {
                    Some(p) => Some(p.to_string()),
                    None => {
                        log::warn!("Ignoring path {:#?} because it is not unicode", p);
                        None
                    }
                }
            })
            .collect()
    }
}
