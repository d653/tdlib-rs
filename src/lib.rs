#![allow(non_snake_case, non_camel_case_types, dead_code)]

include!(concat!(env!("OUT_DIR"), "/tdlib.rs"));

use tdjson::Client;

pub struct Api {
    client: Client,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TaggedSend {
    #[serde(rename = "@extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<Int64>,
    #[serde(flatten)]
    request: TLFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TaggedRecv {
    #[serde(rename = "@extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<Int64>,
    #[serde(flatten)]
    response: TLObject,
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
        let response = self.client.execute(&serialized);
        let deserialized: TLObject = serde_json::from_str(&response).unwrap();
        deserialized
    }

    pub fn send(&mut self, request: impl Into<TLFunction>) {
        let request = request.into();
        let serialized = serde_json::to_string(&request).unwrap();
        self.client.send(&serialized);
    }

    pub fn send_tagged(&mut self, tag:i64, request: impl Into<TLFunction>) {
        let tagged = TaggedSend {
            tag: Some(tag.to_string()),
            request: request.into(),
        };
        let serialized = serde_json::to_string(&tagged).unwrap();
        println!("SEND {}", serialized);
        self.client.send(&serialized);
    }

    pub fn receive(&mut self, timeout: std::time::Duration) -> Option<(Option<i64>, TLObject)> {
        let response = self.client.receive(timeout);
        if let Some(response) = response {
            println!("RECV {}", response);
            let tagged: Result<TaggedRecv, _> = serde_json::from_str(&response);
            if let Ok(tagged) = tagged {
                let tl = tagged.response;
                let tag = tagged.tag;
                let newtag = tag.map(|x| x.parse::<i64>().unwrap());
                return Some((newtag, tl));
            } else {
                panic!("Can not deserialize: {}", response);
            }
        } else {
            None
        }
    }
}
