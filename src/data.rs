use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use serenity::{
    futures::lock::Mutex,
    model::prelude::{ChannelId, UserId},
    prelude::TypeMapKey,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslateRequest {
    pub input: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LlamaRequest {
    pub messages: Vec<LlamaMessage>,
}

#[derive(Debug, Clone)]
pub struct Llama {
    pub channel: ChannelId,
    pub history: Vec<LlamaMessage>,
}

pub struct LlamaData;

impl TypeMapKey for LlamaData {
    type Value = Arc<Mutex<HashMap<ChannelId, Llama>>>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatGPTMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choice {
    pub index: usize,
    pub finish_reason: Option<String>,
    pub message: ChatGPTMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatGPTResponse {
    pub id: String,
    pub object: String,
    pub created: usize,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatGPTRequest {
    pub model: String,
    pub messages: Vec<ChatGPTMessage>,
}

#[derive(Debug, Clone)]
pub struct ChatGPT {
    pub channel: ChannelId,
    pub history: Vec<ChatGPTMessage>,
}

#[derive(Debug, Clone)]
pub struct IndividualChatGPT {
    pub user: UserId,
    pub history: Vec<ChatGPTMessage>,
}

pub struct IndividualChatGPTData;

impl TypeMapKey for IndividualChatGPTData {
    type Value = Arc<Mutex<HashMap<UserId, IndividualChatGPT>>>;
}

pub struct ChatGPTData;

impl TypeMapKey for ChatGPTData {
    type Value = Arc<Mutex<HashMap<ChannelId, ChatGPT>>>;
}
