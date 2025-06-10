mod ict_args;
mod ict_config;

use ict_args::Operation;
use ict_config::load_config;
use ict_server::ict_db::Db;
use ict_server::ict_operations::{
    associate_relay, authorize, clear_relays, delete_device, describe_client, list_clients,
    operate, register, unauthorize,
};
use ict_server::ict_web_axum::start_web_server;
use log::{error, info, LevelFilter};
use std::str::FromStr;

fn main() {
    let args = ict_args::load_args();
    let settings = match load_config(&args.config) {
        Ok(s) => s,
        Err(e) => {
            println!("Error loading config from {}: {}", args.config, e);
            std::process::exit(1);
        }
    };
    env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .filter_level(LevelFilter::from_str(&settings.logs.level).unwrap_or(LevelFilter::Info))
        .init();
    info!("ICT Server starting");
    info!("Using config file: {}", args.config);
    info!("Using DB file: {}", settings.database.path);

    let db = Db::new(&settings.database.path).unwrap_or_else(|e| {
        error!("Failed to open DB with {}", e);
        std::process::exit(1);
    });

    match &args.operation {
        Operation::Register { uuid, public_key } => {
            let secret = register(&db, uuid, public_key).unwrap_or_else(|e| {
                error!("Failed egistration of new client uuid {} with {}",uuid, e);
                std::process::exit(1);
            });
            info!("Successful registration of new client uuid {}, secret is {}", uuid, secret);
        }
        Operation::Authorize { uuid } => {
            match authorize(&db, uuid) {
                Ok(_) => {
                    info!("Successful Authorization of registered client uuid {}",uuid);
                }
                Err(e) => {
                    error!("Failed Authoriztion of registed client uuid {} with {}", uuid, e);
                }
            }
        }
        Operation::Unauthorize { uuid } => {
            match unauthorize(&db, uuid) {
                Ok(_) => {
                    info!("Successful un-authorization of registed client uuid {}",uuid);
                }
                Err(e) => {
                    error!("Failed un-authorization of registed client uuid {} with {}",uuid,e);
                }
            }
        }
        Operation::Delete { uuid } => {
            match delete_device(&db, uuid) {
                Ok(_) => {
                    info!("Successful delete of client uuid {}",uuid);
                }
                Err(e) => {
                    error!("Failed delete of client uuid {} with {}",uuid,e);
                }
            }
        }
        Operation::Operate { uuid, message } => {
            match operate(&db, uuid, &message) {
                Ok(_) => {
                    info!("Successful operate relays of client uuid {}",uuid);
                }
                Err(e) => {
                    error!("Failed operate relays of client uuid {} with {}",uuid,e);
                }
            }
        }
        Operation::ListClients {} => {
            let _ = list_clients(&db);
        }
        Operation::DescribeClient { uuid } => {
            let _ = describe_client(&db, uuid);
        }
        Operation::AssociateRelay { uuid, relay } => {
            match associate_relay(&db, uuid, relay) {
                Ok(_) => {
                    info!("Successful associate relay {} on client {}",relay,uuid);
                }
                Err(e) => {
                    error!("Failed associate relay {} on client {} with {}",relay,uuid,e);
                }
            }
        }
        Operation::ClearRelays { uuid } => {
            match clear_relays(&db, uuid) {
                Ok(_) => {
                    info!("Successful clear relays on client {}",uuid);
                }
                Err(e) => {
                    error!("Successful clear relays on client {} with {}",uuid,e);
                }
            }
        }
        Operation::Serve { port } => {
            info!("Starting server on port {}", port);
            start_web_server(port, &db);
        }
    }
}
