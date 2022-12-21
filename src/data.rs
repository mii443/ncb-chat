use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use serenity::{
    futures::lock::Mutex,
    model::prelude::{ChannelId, Message, UserId},
    prelude::TypeMapKey,
};

#[derive(Debug, Clone)]
pub struct Ping {
    pub channel: ChannelId,
    pub user_id: UserId,
    pub author: UserId,
    pub message: Message,
    pub time: DateTime<Utc>,
    pub args: Vec<String>,
}

pub struct PingData;

impl TypeMapKey for PingData {
    type Value = Arc<Mutex<HashMap<UserId, Ping>>>;
}
