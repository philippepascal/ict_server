use rouille::{Request,Response,router}
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::ict_db::Db;
use crate::ict_operations::{register,operate};

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

pub fn start_web_server(port: &u32, db:&Db) {

    rouille::start_server(format!("0.0.0.0:{}",port), move |request| {
        rouille::log(&request, std::io::stdout(), || {
            ////// won't work... you can pass the path of the db... that's it
        let db = match db.duplicate() {
            Ok(db) => db,
            Err(e) => return Response::empty_404(),
        }

            router!(request,
                (POST) (/register) => {
                    let body: RegisterRequest = match rouille::input::json_input(request) {
                        Ok(data) => data,
                        Err(_) => return Response::text("Invalid JSON").with_status_code(400),
                    };

                    match register(&db,&body.id,&body.pem_public_key) {
                        Ok(secret) => Response::json(&SecretResponse { secret }),
                        Err(e) => Response::text("Registration Failed").with_status_code(400),
                    }
                },

                (POST) (/operate) => {
                    let body: OperateRequest = match rouille::input::json_input(request) {
                        Ok(data) => data,
                        Err(_) => return Response::text("Invalid JSON").with_status_code(400),
                    };

                    match operate(&db,&body.id,&body.encrypted_totp) {
                        Ok(res) => Response::text("Operate successful"),
                        Err(e) => Response::text("Operate Failed").with_status_code(400),
                    }
                },

                _ => {
                    Response::empty_404()
                }
            )
        })
    });
}
