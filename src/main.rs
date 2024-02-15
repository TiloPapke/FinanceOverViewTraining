use std::net::SocketAddr;
use axum::{routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;


#[tokio::main]
async fn main() {
  /*
   // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
*/
    let http = tokio::spawn(http_server_simple());
    let https = tokio::spawn(https_server_simple());

    // Ignore errors.
    let _ = tokio::join!(http,https);
}

async fn http_server_simple() {
    let app = Router::new().route("/", get(|| async { "Hello, world!" }));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener,app).await.unwrap();
}

async fn https_server_simple() {
    let app = Router::new().route("/", get(|| async { "Hello, https world!" }));

    // configure certificate and private key used by https
    let config_async = RustlsConfig::from_pem_file(
        "config/self-signed-certs/cert.pem",
        "config/self-signed-certs/key.pem")
    .await;
    
    let config = config_async.unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3333));

    axum_server::bind_rustls(addr, config)
    .serve(app.into_make_service())
    .await
    .unwrap();

}

