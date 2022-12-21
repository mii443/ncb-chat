use serenity::{model::prelude::Ready, prelude::Context};

pub async fn ready(_: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);
}
