// module schema

use redis::Commands;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::log::logging::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomerDetails {
    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "surname")]
    pub surname: String,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "email")]
    pub email: String,

    #[serde(rename = "mobile")]
    pub mobile: String,
}

#[derive(Clone, Copy, Debug)]
pub struct ImplMessageQueueInterface {}

pub trait MessageQueueInterface {
    // used to interact with container registry (manifest calls)
    fn publish(
        &self,
        log: &Logging,
        data: String,
        host: String,
        topic: String,
    ) -> Result<(), Box<dyn Error>>;
}

impl MessageQueueInterface for ImplMessageQueueInterface {
    fn publish(
        &self,
        log: &Logging,
        json_data: String,
        host: String,
        topic: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // publish to message queue
        let client = redis::Client::open(host).unwrap();
        let mut con = client.get_connection().unwrap();
        let json = serde_json::to_string(&json_data).unwrap();
        log.trace("publishing to queue");
        let _: () = con.publish(topic, json).unwrap();
        Ok(())
    }
}
