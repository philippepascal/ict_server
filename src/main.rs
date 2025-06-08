mod ict_args;
mod ict_config;

use ict_config::load_config;
use ict_server::ict_db::Db;
use ict_server::ict_operations::{associate_relay, authorize, clear_relays, delete_device, describe_client, list_clients, operate, register, unauthorize};

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
            println!(
                "Registering new client with uuid {} and public key {}",
                uuid, public_key
            );
            let secret = register(&db, uuid, public_key).expect("Registration Failure");
            println!("Registration successful, secret: {}", secret);
        }
        Operation::Authorize { uuid } => {
            println!("Authorizing a registered client with uuid {}", uuid);
            match authorize(&db, uuid) {
                Ok(_) => {
                    println!("Authorization successfull");
                }
                Err(e) => {
                    println!("Authoriztion failed {}", e);
                }
            }
        }
        Operation::Unauthorize { uuid } => {
            println!(
                "Unauthorizing a registered client (can be re-authorized) with uuid {}",
                uuid
            );
            match unauthorize(&db, uuid) {
                Ok(_) => {
                    println!("Un-authorization successfull");
                }
                Err(e) => {
                    println!("Un-authoriztion failed {}", e);
                }
            }
        }
        Operation::Delete { uuid } => {
            println!(
                "Deleting a registration (can not be undone) with uuid {}",
                uuid
            );
            match delete_device(&db, uuid) {
                Ok(_) => {
                    println!("delete successfull");
                }
                Err(e) => {
                    println!("delete failed {}", e);
                }
            }
        }
        Operation::Operate { uuid, message } => {
            println!("validate the message passed by client, and operate associated relays with uuid {} and message {}",uuid,message);
            match operate(&db, uuid, &message) {
                Ok(_) => {
                    println!("operate successfull");
                }
                Err(e) => {
                    println!("operate failed {}", e);
                }
            }
        }
        Operation::ListClients {} => {
            println!("listing clients");
            let _ = list_clients(&db);
        }
        Operation::DescribeClient { uuid } => {
            println!("showing status and relays of client with uuid {}", uuid);
            let _ = describe_client(&db, uuid);
        }
        Operation::AssociateRelay { uuid, relay } => {
            println!("add relay {} to client {}", relay, uuid);
            let _ = associate_relay(&db, uuid, relay);
        }
        Operation::ClearRelays { uuid } => {
            println!("removing all relays of client {}", uuid);
            let _ = clear_relays(&db, uuid);
        }
        Operation::Serve { port } => {
            println!("Starting server on port {}", port);
        }
    }
}
