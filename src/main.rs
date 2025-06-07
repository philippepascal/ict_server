mod ict_args;
mod ict_config;

use ict_config::load_config;
use ict_server::ict_db::Db;
use ict_server::ict_operations::{ register};


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
            println!("Registering new client with uuid {} and public key {}",uuid,public_key);
            let secret = register(db, uuid, public_key).expect("Registration Failure");
            println!("Registration successful, secret: {}",secret);
        }
        Operation::Authorize { uuid } => {
            println!("Authorizing a registered client with uuid {}",uuid);
        }
        Operation::Unauthorize { uuid } => {
            println!("Unauthorizing a registered client (can be re-authorized) with uuid {}",uuid);
        }
        Operation::Delete { uuid } => {
            println!("Deleting a registration (can not be undone) with uuid {}",uuid);
        }
        Operation::Operate { uuid, message } => {
            println!("validate the message passed by client, and operate associated relays with uuid {} and message {}",uuid,message);
        }
        Operation::ListClients {} => {
            println!("listing clients");
        }
        Operation::DescribeClient { uuid } => {
            println!("showing status and relays of client with uuid {}",uuid);
        }
        Operation::AssociateRelay { uuid, relay } => {
            println!("add relay {} to client {}", relay, uuid);
        }
        Operation::ClearRelays { uuid } => {
            println!("removing all relays of client {}", uuid);
        }
        Operation::Serve { port } => {
            println!("Starting server on port {}", port);
        }
    }
}
