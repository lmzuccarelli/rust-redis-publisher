use hyper::server::conn::Http;
use hyper::service::service_fn;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// define local modules
mod api;
mod handlers;
mod log;

// use local modules
use handlers::publisher::*;
use log::logging::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let log = &Logging {
        log_level: Level::TRACE,
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;
    log.info(&format!("Listening on http://{}", addr));
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = Http::new()
                .http1_only(true)
                .http1_keep_alive(true)
                .serve_connection(stream, service_fn(process_payload))
                .await
            {
                log.error(&format!("Error serving connection: {:}", err));
            }
        });
    }
}
