mod group;
mod query;
mod types;

pub use types::*;

#[derive(Debug, Clone)]
pub struct Client {
    client: cloudwatchlogs::Client,
}

impl Client {
    pub fn new(client: cloudwatchlogs::Client) -> Self {
        Self { client }
    }
}
