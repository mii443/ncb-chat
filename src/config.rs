use std::sync::Arc;

use serde::Deserialize;
use serenity::{futures::lock::Mutex, prelude::TypeMapKey};

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub application_id: u64,
    pub llama_url: String,
    pub openai_key: String,
    pub chatgpt_allows: Vec<i64>,
    pub chatgpt_forums: Vec<i64>,
}

pub struct ConfigData;

impl TypeMapKey for ConfigData {
    type Value = Arc<Mutex<Config>>;
}
