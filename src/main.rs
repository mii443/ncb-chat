mod config;
mod data;
mod event_handler;
mod events;

use std::{collections::HashMap, env, sync::Arc};

use config::{Config, ConfigData};
use data::{ChatGPTData, IndividualChatGPTData, LlamaData};
use event_handler::Handler;
use serenity::{
    client::Client, framework::StandardFramework, futures::lock::Mutex, prelude::GatewayIntents,
};

/// Create discord client
///
/// Example:
/// ```rust
/// let client = create_client("!", "BOT_TOKEN", 123456789123456789).await;
///
/// client.start().await;
/// ```
async fn create_client(prefix: &str, token: &str, id: u64) -> Result<Client, serenity::Error> {
    let framework = StandardFramework::new().configure(|c| c.with_whitespace(true).prefix(prefix));

    Client::builder(token, GatewayIntents::all())
        .event_handler(Handler)
        .application_id(id)
        .framework(framework)
        .await
}

#[tokio::main]
async fn main() {
    // Load config
    let config = {
        let config = std::fs::read_to_string("./config.toml");
        if let Ok(config) = config {
            toml::from_str::<Config>(&config).expect("Cannot load config file.")
        } else {
            let token = env::var("NCB_TOKEN").unwrap();
            let application_id = env::var("NCB_APP_ID").unwrap();
            let llama_url = env::var("LLAMA_URL").unwrap();
            let openai_key = env::var("OPENAI_KEY").unwrap();
            let chatgpt_allows = env::var("CHATGPT_ALLOWS").unwrap().to_string();
            let chatgpt_allows: Vec<i64> = chatgpt_allows
                .split(",")
                .map(|f| i64::from_str_radix(f, 10).unwrap())
                .collect();
            let chatgpt_forums = env::var("CHATGPT_FORUMS").unwrap().to_string();
            let chatgpt_forums: Vec<i64> = chatgpt_forums
                .split(",")
                .map(|f| i64::from_str_radix(f, 10).unwrap())
                .collect();

            Config {
                token,
                application_id: u64::from_str_radix(&application_id, 10).unwrap(),
                llama_url,
                openai_key,
                chatgpt_allows,
                chatgpt_forums,
            }
        }
    };

    // Create discord client
    let mut client = create_client("p.", &config.token, config.application_id)
        .await
        .expect("Err creating client");

    // Create TTS storage
    {
        let mut data = client.data.write().await;
        data.insert::<LlamaData>(Arc::new(Mutex::new(HashMap::default())));
        data.insert::<ChatGPTData>(Arc::new(Mutex::new(HashMap::default())));
        data.insert::<IndividualChatGPTData>(Arc::new(Mutex::new(HashMap::default())));
        data.insert::<ConfigData>(Arc::new(Mutex::new(config)));
    }

    // Run client
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
