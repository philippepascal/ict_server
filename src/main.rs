use clap::Parser;

/// MyApp – does something cool with a config file
#[derive(Parser, Debug)]
#[command(name = "ict_server")]
#[command(about = "Runs the ICT server", long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(
        short,
        long,
        value_name = "PATH-TO-FILE",
        default_value = "configs/ict_server.toml"
    )]
    config: String,
}

fn main() {
    let args = Args::parse();
    println!("Using config file: {}", args.config);
}
