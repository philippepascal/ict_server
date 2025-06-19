use crate::ict_db::Db;
use crate::ict_operations::{operate, register};
use crate::ict_config::Settings;
use log::{info,error};
use rouille::{router, Response};
use serde::{Deserialize, Serialize};
use std::time::Instant;

// Simulate a DB with ID → (public_key, secret)
// type Db = Arc<Mutex<HashMap<String, (String, String)>>>;

#[derive(Deserialize)]
struct RegisterRequest {
    id: String,
    pem_public_key: String,
}

#[derive(Deserialize)]
struct OperateRequest {
    id: String,
    totp_message: String,
    signature: String,
}

#[derive(Serialize)]
struct SecretResponse {
    encrypted_secret: String,
}

pub fn start_web_server(port: &u32, db: &Db, settings: Settings) {
    let db_path2 = db.path.clone();
    let db_sha = db.totp_sha.clone();
    rouille::start_server(format!("0.0.0.0:{}", port), move |request| {
            let start = Instant::now();
            let db2 = match Db::newg(db_path2.clone(),db_sha.clone()) {
                Ok(db2) => db2,
                Err(e) => {
                    error!("Could not instantiate Db while processing request with {}",e);
                    return Response::text("Server Failure").with_status_code(400)
                },
            };
            let response = router!(request,
                (POST) (/register) => {
                    let body: RegisterRequest = match rouille::input::json_input(request) {
                        Ok(data) => data,
                        Err(e) => {
                            error!("Could not parse the body {}",e);
                            return Response::text("Invalid JSON").with_status_code(400)
                        },
                    };
                    match register(&db2,&body.id,&body.pem_public_key) {
                            Ok(encrypted_secret) => {
                                info!("Successful register during web request with uuid {}",&body.id);
                                Response::json(&SecretResponse { encrypted_secret })
                            },
                            Err(e) => {
                                error!("Failed register during web request uuid {} with {}",&body.id,e);
                                Response::text("Registration Failed").with_status_code(400)
                            },
                        }
                },

                (POST) (/operate) => {
                    let body: OperateRequest = match rouille::input::json_input(request) {
                        Ok(data) => data,
                        Err(_) => return Response::text("Invalid JSON").with_status_code(400),
                    };

                    match operate(&db2,&body.id,&body.totp_message,&body.signature, &settings.pi.close_duration) {
                            Ok(_) => {
                                info!("Successful operate during web request with uuid {}",&body.id);
                                Response::text("Operate successful")
                            },
                            Err(e) => {
                                error!("Failed operate during web request uuid {} with {}",&body.id,e);
                                Response::text("Operate Failed").with_status_code(400)
                            },
                         }
                },

                (GET) (/stats) => {
                    Response::text("Server up and running").with_status_code(200)
                },

                _ => {
                    Response::empty_404()
                }
            );
            let duration = start.elapsed();
            info!("{} {} from:{} code:{} {:.2?}", request.method(), request.url(), request.remote_addr(), response.status_code, duration);
            response
        })
}
