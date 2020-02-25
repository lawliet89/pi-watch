mod stat;

use snafu::ResultExt;

#[derive(Debug, snafu::Snafu)]
pub enum Error {
    /// Sensor Error
    Stat { source: stat::Error },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let stat = stat::Stat::new();

    for reading in stat.temperature().await {
        println!("{:#?}", reading)
    }
    // println!("{}", stat.temperature()?);

    Ok(())
}
