use serenity::{model::prelude::command::Command, model::prelude::Ready, prelude::Context};

pub async fn ready(ctx: Context, ready: Ready) {
    println!("{} is connected!", ready.user.name);

    let mut cosmo = true;
    let mut chatgpt = true;

    for command in Command::get_global_application_commands(&ctx.http)
        .await
        .unwrap()
    {
        if command.name == "cosmo" {
            cosmo = false;
        }
        if command.name == "chatgpt" {
            chatgpt = false;
        }
    }

    if cosmo {
        Command::create_global_application_command(&ctx.http, |command| {
            command.name("cosmo").description("Start cosmo chat.")
        })
        .await
        .unwrap();
    }
    if chatgpt {
        Command::create_global_application_command(&ctx.http, |command| {
            command.name("chatgpt").description("Start ChatGPT chat.")
        })
        .await
        .unwrap();
    }
}
