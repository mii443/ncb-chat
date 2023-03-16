use serenity::model::prelude::interaction::Interaction;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{channel::GuildChannel, channel::Message, gateway::Ready},
};

use crate::config::ConfigData;
use crate::data::*;
use crate::events;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            let name = &*command.data.name;
            if name == "llama" {
                let message = command
                    .channel_id
                    .send_message(&ctx.http, |f| f.content("LLaMa Chat thread"))
                    .await
                    .unwrap();
                let id = command
                    .channel_id
                    .create_public_thread(&ctx.http, message, |f| {
                        f.name("LLaMa Chat").auto_archive_duration(60)
                    })
                    .await
                    .unwrap()
                    .id;
                let llama = Llama {
                    channel: id,
                    history: vec![],
                };
                let storage_lock = {
                    let data_read = ctx.data.read().await;
                    data_read
                        .get::<LlamaData>()
                        .expect("Cannot get TTSStorage")
                        .clone()
                };
                storage_lock.lock().await.insert(id, llama);
            }

            if name == "chatgpt" {
                let cs = {
                    let data_read = ctx.data.read().await;
                    data_read
                        .get::<ConfigData>()
                        .expect("Cannot get ConfigData")
                        .clone()
                };
                let config = cs.lock().await;

                if !config.chatgpt_allows.contains(&(command.user.id.0 as i64)) {
                    return;
                }

                let message = command
                    .channel_id
                    .send_message(&ctx.http, |f| f.content("ChatGPT thread"))
                    .await
                    .unwrap();
                let id = command
                    .channel_id
                    .create_public_thread(&ctx.http, message, |f| {
                        f.name("ChatGPT").auto_archive_duration(60)
                    })
                    .await
                    .unwrap()
                    .id;
                let chatgpt = ChatGPT {
                    channel: id,
                    history: vec![],
                };
                let storage_lock = {
                    let data_read = ctx.data.read().await;
                    data_read
                        .get::<ChatGPTData>()
                        .expect("Cannot get TTSStorage")
                        .clone()
                };
                storage_lock.lock().await.insert(id, chatgpt);
            }
        }
    }
    async fn message(&self, ctx: Context, message: Message) {
        events::message_receive::message(ctx, message).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        events::ready::ready(ctx, ready).await
    }
}
