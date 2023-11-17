use hyper::body;
use hyper::{Body, Error, Method, Request, Response};
use redis::Commands;
use std::env;

use crate::api::schema::*;
use crate::log::logging::*;

/// handler - reads json as input
pub async fn process_payload(req: Request<Body>) -> Result<Response<Body>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/publish") => {
            // read envars
            let h = env::var("REDIS_HOST").is_ok();
            let host = match h {
                true => env::var("REDIS_HOST").unwrap(),
                false => String::from("redis://127.0.0.1:6379"),
            };
            let tp = env::var("TOPIC").is_ok();
            let topic = match tp {
                true => env::var("TOPIC").unwrap(),
                false => String::from("test"),
            };
            let res = env::var("LOG_LEVEL").is_ok();
            // create a logging instance
            let lvl = match res {
                true => match env::var("LOG_LEVEL").unwrap().as_str() {
                    "info" => Level::INFO,
                    "debug" => Level::DEBUG,
                    "trace" => Level::TRACE,
                    _ => Level::INFO,
                },
                false => Level::INFO,
            };

            let log = &Logging { log_level: lvl };
            let payload = body::to_bytes(req.into_body()).await?;
            log.info(&format!("payload {:#?}", payload));
            let obj = serde_json::from_slice::<CustomerDetails>(&payload).unwrap();
            let json_data = serde_json::to_string(&obj).unwrap();
            log.info(&format!("publish to topic {:#?}", topic));
            // publish to message queue
            let client = redis::Client::open(host).unwrap();
            let mut con = client.get_connection().unwrap();
            let json = serde_json::to_string(&json_data).unwrap();
            let _: () = con.publish(topic, json).unwrap();

            Ok(Response::new(Body::from(
                "json data successfully published to message queue",
            )))
        }
        // health endpoint
        (&Method::GET, "/isalive") => Ok(Response::new(Body::from("ok"))),
        // all other routes
        _ => Ok(Response::new(Body::from(
            "ensure you post to the /publish endpoint with valid json",
        ))),
    }
}
