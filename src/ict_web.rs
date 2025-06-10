use crate::ict_db::Db;
use crate::ict_operations::{operate, register};
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
    encrypted_totp: String,
}

#[derive(Serialize)]
struct SecretResponse {
    secret: String,
}

pub fn start_web_server(port: &u32, db: &Db) {
    let db_path2 = db.path.clone();
    rouille::start_server(format!("0.0.0.0:{}", port), move |request| {
            let start = Instant::now();
            let db2 = match Db::newg(db_path2.clone()) {
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
                        Err(_) => return Response::text("Invalid JSON").with_status_code(400),
                    };
                    match register(&db2,&body.id,&body.pem_public_key) {
                            Ok(secret) => {
                                info!("Successful register during web request with uuid {}",&body.id);
                                Response::json(&SecretResponse { secret })
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

                    match operate(&db2,&body.id,&body.encrypted_totp) {
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
