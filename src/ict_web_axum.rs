use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use log::info;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

#[tokio::main]
pub async fn start_web_server(port: &u32,tls_path: &str) {
    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(tls_path)
            .join("cert.pem"),
        PathBuf::from(tls_path)
            .join("key.pem"),
    )
    .await
    .unwrap();
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    axum_server::bind_rustls(SocketAddr::from(([0,0,0,0],*port as u16)), config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    info!("root GET");
    "Hello, World!"
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
