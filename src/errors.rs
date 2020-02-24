#[derive(Debug, snafu::Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("IO Error while {}: {}", context, source))]
    IOError {
        source: std::io::Error,
        context: &'static str,
    },
}
