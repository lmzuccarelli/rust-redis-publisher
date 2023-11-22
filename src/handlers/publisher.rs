use hyper::body;
use hyper::{Body, Error, Method, Request, Response};
//use redis::Commands;
use std::env;

use crate::api::schema::*;
use crate::log::logging::*;

/// handler - reads json as input
pub async fn process_payload<T: MessageQueueInterface>(
    req: Request<Body>,
    log: &Logging,
    q: T,
) -> Result<Response<Body>, Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/publish") => {
            // read envars
            let h = env::var("REDIS_HOST").is_ok();
            let host = match h {
                true => env::var("REDIS_HOST").unwrap(),
                false => {
                    log.warn("envar REDIS_HOST is not set (using default)");
                    String::from("redis://127.0.0.1:6379")
                }
            };
            let tp = env::var("TOPIC").is_ok();
            let topic = match tp {
                true => env::var("TOPIC").unwrap(),
                false => String::from("test"),
            };
            let payload = body::to_bytes(req.into_body()).await?;
            log.info(&format!("payload {:#?}", payload));
            let obj = serde_json::from_slice::<CustomerDetails>(&payload).unwrap();
            let json_data = serde_json::to_string(&obj).unwrap();
            log.debug(&format!("publish to topic {:#?}", topic));

            q.publish(log, json_data, host, topic).unwrap();

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
#[cfg(test)]
mod tests {
    // this brings everything from parent's scope into this scope
    use super::*;
    use hyper::{Body, Request, Uri};
    use serial_test::serial;
    use std::fs;
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e).unwrap()
        };
    }
    struct Mock {}
    impl MessageQueueInterface for Mock {
        fn publish(
            &self,
            log: &Logging,
            json_data: String,
            _host: String,
            _topic: String,
        ) -> Result<(), Box<dyn std::error::Error>> {
            log.info("testing queue publish");
            log.info(&format!("data {:#?}", json_data));
            Ok(())
        }
    }
    #[test]
    #[serial]
    fn test_handler_post_setvars_pass() {
        let log = &Logging {
            log_level: Level::TRACE,
        };
        env::remove_var("REDIS_HOST");
        env::remove_var("TOPIC");
        let tst = Mock {};
        let payload = fs::read_to_string("./payload.json").expect("should read payload.json file");
        let req = Request::new(Body::from(payload));
        let uri = "https://www.rust-lang.org/publish".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::POST;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
    }
    #[test]
    #[serial]
    fn test_handler_post_novars_pass() {
        let log = &Logging {
            log_level: Level::TRACE,
        };
        let tst = Mock {};
        env::set_var("REDIS_HOST", "redis://test");
        env::set_var("TOPIC", "test");
        let payload = fs::read_to_string("./payload.json").expect("should read payload.json file");
        let req = Request::new(Body::from(payload));
        let uri = "https://www.rust-lang.org/publish".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::POST;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
    }
    #[test]
    #[serial]
    fn test_handler_get_pass() {
        let log = &Logging {
            log_level: Level::INFO,
        };
        let tst = Mock {};
        let req = Request::new(Body::from("ok"));
        let uri = "https://www.rust-lang.org/isalive".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::GET;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
        env::remove_var("REDIS_HOST");
        env::remove_var("TOPIC");
    }
    #[test]
    fn test_handler_other_pass() {
        let log = &Logging {
            log_level: Level::INFO,
        };
        let tst = Mock {};
        let req = Request::new(Body::from("please check your payload"));
        let uri = "https://www.rust-lang.org/".parse::<Uri>().unwrap();
        let (mut parts, body) = req.into_parts();
        parts.method = Method::PUT;
        parts.uri = uri;
        let request = Request::from_parts(parts, body);
        aw!(process_payload(request, log, tst));
    }
}
