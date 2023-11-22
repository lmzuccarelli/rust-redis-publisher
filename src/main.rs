use api::schema::ImplMessageQueueInterface;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use std::env;
use std::net::SocketAddr;
use std::process;
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

    // exit process as SERVER_PORT envar is not set
    if env::var("SERVER_PORT").is_err() {
        log.error("envar SERVER_PORT (mandatory) is not set");
        process::exit(-1);
    }

    let port = env::var("SERVER_PORT").unwrap().parse::<u16>().unwrap();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    log.info(&format!("Listening on http://{}", addr));
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            let real = ImplMessageQueueInterface {};
            if let Err(err) = Http::new()
                .http1_only(true)
                .http1_keep_alive(true)
                .serve_connection(
                    stream,
                    service_fn(move |req| process_payload(req, log, real)),
                )
                .await
            {
                log.error(&format!("Error serving connection: {:}", err));
            }
        });
    }
}
