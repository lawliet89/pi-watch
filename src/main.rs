mod errors;
mod stat;

#[tokio::main]
async fn main() -> Result<(), errors::Error> {
    env_logger::init();

    let stat = stat::Stat::new();
    println!("{:#?}", stat::Stat::sensors_temperature());
    println!("{}", stat.temperature()?);

    Ok(())
}
