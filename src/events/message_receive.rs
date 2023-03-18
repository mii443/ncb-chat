use chrono::Utc;
use serenity::{
    http::CacheHttp,
    model::prelude::{GuildChannel, Message, MessageId, ReactionType},
    prelude::Context,
};
use url::Url;

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
        let mut history = llama.history.clone();

        if message.content.trim() == "reset".to_string() {
            llama.history = vec![];
            llama_storage.insert(llama.channel, llama.clone());
            message
                .reply(&ctx.http, "会話履歴をリセットしました。")
                .await
                .unwrap();
            return;
        }

        let typing = message.channel_id.start_typing(&ctx.http).unwrap();
        //let text = translate_ja_en(message.content.clone()).await;
        let text = message.content.clone();
        println!("{}", text);
        history.push(LlamaMessage {
            role: "user".to_string(),
            content: text,
        });
        llama.history = history.clone();
        llama_storage.insert(llama.channel, llama.clone());

        let request = LlamaRequest {
            messages: history.clone(),
        };

        let (mut socket, response) = tungstenite::connect(Url::parse("ws://192.168.0.19:18080/").unwrap()).expect("Can't connect to websocket server");

        socket.write_message(tungstenite::Message::Text(serde_json::to_string(&request).unwrap().into())).unwrap();

        let mut buffer = String::default();
        let rate = 3;
        let mut count = 0;

        let mut response_message: Message = if let Ok(s) = socket.read_message() {
            if let tungstenite::Message::Text(msg) = s {
                buffer = buffer + &msg.to_string();
                message.channel_id.send_message(&ctx.http, |f| f.content(msg)).await.unwrap()
            } else {
                panic!("cannot read message");
            }
        } else {
            panic!("cannot read message");
        };

        loop {
            if let Ok(s) = socket.read_message() {
                match s {
                    tungstenite::Message::Text(msg) => {
                        buffer = buffer + &msg.to_string();
                        println!("{}", msg.to_string());

                        if count == rate {
                            response_message.edit(&ctx.http, |f| f.content(buffer.clone())).await.unwrap();
                            count = 0;
                        }
                        count += 1;
                    }
                    _ => {
                        break;
                    }
                }
            }
        }

        response_message.edit(&ctx.http, |f| f.content(buffer.clone())).await.unwrap();
        response_message.react(&ctx.http, ReactionType::Unicode("✅".to_string())).await.unwrap();

        typing.stop().unwrap();
/*
        let client = reqwest::Client::new();
        match client
            .get("http://localhost:18080/")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .await
        {
            Ok(ok) => {
                let response_en = ok.text().await.expect("ERROR").trim().to_string();
                //let response = translate_en_ja(response_en.clone()).await;
                //println!("JA: {}", response);
                println!("EN: {}", response_en);

                history.push(LlamaMessage {
                    role: "ai".to_string(),
                    content: response_en.clone(),
                });

                llama.history = history;

                llama_storage.insert(llama.channel, llama.clone());

                message
                    .channel_id
                    .send_message(&ctx.http, |f| f.content(response_en))
                    .await
                    .unwrap();
            }
            Err(err) => {
                panic!("Error")
            }
        } */
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
