mod ict_args;
mod ict_config;
mod ict_db;

use ict_config::load_config;
use ict_db::Db;

fn main() {
    let args = ict_args::load_args();
    println!("Using config file: {}", args.config);

    let settings = match load_config(&args.config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading config from {}: {}", args.config, e);
            std::process::exit(1);
        }
    };
    println!("Using DB file: {}", settings.database.path);

    let db = Db::new(&settings.database.path).expect("Failed to open DB");

    println!("Total devices: {}", db.count_devices().unwrap());
    db.print_all_devices().unwrap();
}
