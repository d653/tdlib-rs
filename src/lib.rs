#![allow(non_snake_case, non_camel_case_types, dead_code)]

include!(concat!(env!("OUT_DIR"), "/tdlib.rs"));

use tdjson::Client;

pub struct Api {
    client: Client,
}

impl Api {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn execute(&mut self, request: impl Into<TLFunction>) -> TLObject {
        let request = request.into();
        let serialized = serde_json::to_string(&request).unwrap();
        let response = self.client.execute(&serialized).unwrap();
        let deserialized: TLObject = serde_json::from_str(&response).unwrap();
        deserialized
    }

    pub fn send(&mut self, request: impl Into<TLFunction>) {
        let request = request.into();
        let serialized = serde_json::to_string(&request).unwrap();
        self.client.send(&serialized);
    }

    pub fn receive(&mut self, timeout: std::time::Duration) -> Option<TLObject> {
        let response = self.client.receive(timeout);
        if let Some(response) = response {
            let response = response.unwrap();
            let deserialized: TLObject = serde_json::from_str(&response).unwrap();
            Some(deserialized)
        } else {
            None
        }
    }
}
