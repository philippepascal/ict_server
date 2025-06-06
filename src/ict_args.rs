use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ict_server")]
#[command(about = "Runs the ICT server", long_about = None)]
pub struct Args {
    /// Path to the configuration file
    #[arg(
        short,
        long,
        value_name = "PATH-TO-FILE",
        default_value = "configs/ict_server.toml"
    )]
    pub config: String,
}

pub fn load_args() -> Args {
    Args::parse()
}