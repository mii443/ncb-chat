use chrono::Utc;
use serenity::{
    http::CacheHttp,
    model::prelude::{GuildChannel, Message},
    prelude::Context,
};

use crate::{
    config::*,
    data::{
        ChatGPT, ChatGPTData, ChatGPTMessage, ChatGPTRequest, ChatGPTResponse, IndividualChatGPT,
        IndividualChatGPTData, Llama, LlamaData, LlamaMessage, LlamaRequest, TranslateRequest,
    },
};

async fn translate_en_ja(input: String) -> String {
    let request = TranslateRequest { input };
    let client = reqwest::Client::new();
    match client
        .post("http://localhost:8008/translate/enja")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&request).unwrap())
        .send()
        .await
    {
        Ok(ok) => ok.text().await.expect("ERROR"),
        Err(err) => {
            panic!("Error")
        }
    }
}

async fn translate_ja_en(input: String) -> String {
    let request = TranslateRequest { input };
    let client = reqwest::Client::new();
    match client
        .post("http://localhost:8008/translate/jaen")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&request).unwrap())
        .send()
        .await
    {
        Ok(ok) => ok.text().await.expect("ERROR"),
        Err(err) => {
            panic!("Error")
        }
    }
}

async fn chatgpt_request(input: ChatGPTRequest, key: String) -> ChatGPTResponse {
    let client = reqwest::Client::new();
    match client
        .post("https://api.openai.com/v1/chat/completions")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", key))
        .body(serde_json::to_string(&input).unwrap())
        .send()
        .await
    {
        Ok(ok) => {
            let text = ok.text().await.unwrap();
            println!("{}", text.clone());
            let response: ChatGPTResponse = serde_json::from_str(&text).unwrap();
            response
        }
        Err(err) => {
            panic!("Error")
        }
    }
}

fn split_string_into_chunks(s: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let len = s.len();

    while start < len {
        let end = if start + chunk_size < len {
            start + chunk_size
        } else {
            len
        };
        chunks.push(s[start..end].to_string());
        start = end;
    }

    chunks
}

pub async fn llama(ctx: Context, message: Message) {}

pub async fn message(ctx: Context, message: Message) {
    if message.author.bot {
        return;
    }

    if message.content.starts_with(";") {
        message
            .reply(&ctx.http, "スキップしました。")
            .await
            .unwrap();
        return;
    }

    let guild_id = message.guild(&ctx.cache);

    if let None = guild_id {
        return;
    }

    println!("Event received: {}", message.content);
    let config_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ConfigData>()
            .expect("Cannot get Config")
            .clone()
    };

    let config = config_lock.lock().await;

    let llama_storage_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<LlamaData>()
            .expect("Cannot get LlamaData")
            .clone()
    };

    let mut llama_storage = llama_storage_lock.lock().await;

    if let Some(mut llama) = llama_storage.clone().get_mut(&message.channel_id) {
        let typing = message.channel_id.start_typing(&ctx.http).unwrap();
        let mut history = llama.history.clone();
        let text = translate_ja_en(message.content.clone()).await;
        println!("{}", text);
        history.push(LlamaMessage {
            role: "user".to_string(),
            content: text,
        });
        llama.history = history.clone();
        llama_storage.insert(llama.channel, llama.clone());

        let request = LlamaRequest { messages: history };

        let client = reqwest::Client::new();
        match client
            .post("http://localhost:18080/")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .await
        {
            Ok(ok) => {
                let response = ok.text().await.expect("ERROR");
                let response = translate_en_ja(response).await;
                println!("{}", response);
                message
                    .channel_id
                    .send_message(&ctx.http, |f| f.content(response))
                    .await
                    .unwrap();
            }
            Err(err) => {
                panic!("Error")
            }
        }

        typing.stop().unwrap();
    }

    let chatgpt_storage_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<ChatGPTData>()
            .expect("Cannot get ChatGPTData")
            .clone()
    };

    let mut chatgpt_storage = chatgpt_storage_lock.lock().await;

    // forum
    let channel = message.channel(&ctx.http).await.unwrap();
    if let Some(guild_channel) = channel.clone().guild() {
        if let Some(parent) = guild_channel.parent_id {
            if chatgpt_storage
                .clone()
                .get_mut(&message.channel_id)
                .is_none()
                && config.chatgpt_forums.contains(&(parent.0 as i64))
            {
                chatgpt_storage.insert(
                    guild_channel.id,
                    ChatGPT {
                        channel: channel.id(),
                        history: vec![],
                    },
                );
            }
        }
    }

    if let Some(mut chatgpt) = chatgpt_storage.clone().get_mut(&message.channel_id) {
        if message.content == "reset".to_string() {
            chatgpt.history = vec![];
            chatgpt_storage.insert(chatgpt.channel, chatgpt.clone());
            message
                .channel_id
                .send_message(&ctx.http, |f| f.content("会話履歴をリセットしました。"))
                .await
                .unwrap();
            return;
        }

        let typing = message.channel_id.start_typing(&ctx.http).unwrap();
        let mut history = chatgpt.history.clone();

        history.push(ChatGPTMessage {
            role: "user".to_string(),
            content: message.content.clone(),
        });

        let request = ChatGPTRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: history.clone(),
        };

        let response = chatgpt_request(request, config.openai_key.clone()).await;

        history.push(response.choices[0].message.clone());

        chatgpt.history = history.clone();

        chatgpt_storage.insert(chatgpt.channel, chatgpt.clone());

        println!("Tokens: {:?}", response.usage.total_tokens);
        let responses = format!(
            "{}\ntokens: {}/4096",
            response.choices[0].message.content.clone(),
            response.usage.total_tokens
        );
        let responses = split_string_into_chunks(&responses, 2000);

        let l = responses.len();
        for response in responses {
            if l > 1 {
                message.reply(&ctx.http, response.replace("```", "")).await;
            } else {
                message.reply(&ctx.http, response).await;
            }
        }

        typing.stop().unwrap();
    }

    let individual_chatgpt_storage_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<IndividualChatGPTData>()
            .expect("Cannot get IndividualChatGPTData")
            .clone()
    };

    let mut individual_storage = individual_chatgpt_storage_lock.lock().await;

    let bot_id = format!("<@{}>", config.application_id);
    if message.content.starts_with(&bot_id) {
        if !config
            .chatgpt_allows
            .contains(&(message.author.id.0 as i64))
        {
            message
                .reply(&ctx.http, "権限がありません。")
                .await
                .unwrap();
            return;
        }

        let text = message.content.replacen(&bot_id, "", 1);

        let mut tmp = IndividualChatGPT {
            user: message.author.id.clone(),
            history: vec![],
        };

        let storage_tmp = individual_storage.clone();
        let chatgpt = storage_tmp.get(&message.author.id.clone()).unwrap_or(&tmp);
        let mut chatgpt = chatgpt.clone();

        if text.trim() == "reset".to_string() {
            individual_storage.insert(message.author.id.clone(), tmp);
            message
                .reply(&ctx.http, "会話履歴をリセットしました。")
                .await
                .unwrap();
            return;
        }

        let typing = message.channel_id.start_typing(&ctx.http).unwrap();
        let mut history = chatgpt.history.clone();

        history.push(ChatGPTMessage {
            role: "user".to_string(),
            content: message.content.clone(),
        });

        let request = ChatGPTRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: history.clone(),
        };

        let response = chatgpt_request(request, config.openai_key.clone()).await;

        history.push(response.choices[0].message.clone());

        chatgpt.history = history.clone();

        individual_storage.insert(chatgpt.user.clone(), chatgpt.clone());

        println!("Tokens: {:?}", response.usage.total_tokens);
        let responses = format!(
            "{}\ntokens: {}/4096",
            response.choices[0].message.content.clone(),
            response.usage.total_tokens
        );
        let responses = split_string_into_chunks(&responses, 2000);

        let l = responses.len();
        for response in responses {
            if l > 1 {
                message.reply(&ctx.http, response.replace("```", "")).await;
            } else {
                message.reply(&ctx.http, response).await;
            }
        }

        typing.stop().unwrap();
    }
}
