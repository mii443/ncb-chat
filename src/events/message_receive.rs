use chrono::Utc;
use serenity::{model::prelude::Message, prelude::Context};

use crate::data::{PingData, Ping};

pub async fn message(ctx: Context, message: Message) {
    if message.author.bot {
        return;
    }

    let guild_id = message.guild(&ctx.cache);

    if let None = guild_id {
        return;
    }

    let storage_lock = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<PingData>()
            .expect("Cannot get PingData")
            .clone()
    };

    if message.mentions.len() == 1 {
        let user = message.mentions.first().unwrap();
        let m = format!("PING {} ({}) 56(84) bytes of data.", user.name, user.id.0);
        let ping_message = message.reply(&ctx.http, m).await.unwrap();

        let ping = Ping {
            channel: message.channel_id,
            user_id: user.id,
            author: message.author.id,
            time: Utc::now(),
            message: ping_message,
            args: vec![]
        };

        let mut storage = storage_lock.lock().await;
        storage.insert(user.id, ping.clone());
    }

    {
        let mut storage = storage_lock.lock().await;
        if !storage.contains_key(&message.author.id) {
            return;
        }
        let ping = storage.get_mut(&message.author.id).unwrap();

        if ping.channel == message.channel_id {
            let user = ping.user_id.to_user(&ctx.http).await.unwrap();
            let time = Utc::now() - ping.time;
            ping.message.edit(&ctx.http, |f| f.content(format!("--- {} ping statistics ---\n1 packets transmitted, 1 received, 0% packet loss, time {}ms", user.name, time.num_milliseconds()))).await.unwrap();
            message.channel_id.send_message(&ctx.http, |f| f.content(format!("<@{}>", ping.author.0))).await.unwrap();
            storage.remove(&message.author.id);
        }
    }
}
