mod ict_args;
mod ict_config;

use ict_config::load_config;
use ict_server::ict_db::Db;

use crate::ict_args::Operation;

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

    match &args.operation {
        Operation::Register { uuid, public_key } => {
            println!("Registering new client");
        }
        Operation::Authorize { uuid } => {
            println!("Authorizing a registered client");
        }
        Operation::Unauthorize { uuid } => {
            println!("Unauthorizing a registered client (can be re-authorized)");
        }
        Operation::Delete { uuid } => {
            println!("Deleting a registration (can not be undone)");
        }
        Operation::Operate { uuid, message } => {
            println!("validate the message passed by client, and operate associated relays");
        }
        Operation::List_Clients {} => {
            println!("listing clients");
        }
        Operation::Describe_Client { uuid } => {
            println!("showing status and relays of client");
        }
        Operation::Associate_Relay { uuid, relay } => {
            println!("add relay {} to client {}", relay, uuid);
        }
        Operation::Clear_Relays { uuid } => {
            println!("removing all relays of client {}", uuid);
        }
        Operation::Serve { port } => {
            println!("Starting server on port {}", port);
        }
    }
}
