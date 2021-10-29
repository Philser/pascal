use std::{
    collections::{HashMap, HashSet},
    env,
};

use anyhow::Context as AnyhowCtx;
use log::{error, info};
use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    framework::StandardFramework,
    http::Http,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use songbird::SerenityInit;

use crate::commands::GENERAL_GROUP;
use crate::{commands::help::HELP, utils::error::handle_error};

mod commands;
mod utils;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Pascal starting...");

    let mut conf = config::Config::default();
    if let Err(err) = conf.merge(config::Environment::with_prefix("")) {
        error!("Unable to load ENV variables: {}", err);
        return;
    }

    conf.get_str("config").unwrap();
    // If no custom conf file is specified, look for conf.yml
    if let Ok(config_file) = conf.get_str("config") {
        if let Err(err) = conf.merge(config::File::with_name(&config_file)) {
            error!("Unable to load config file: {}", err);
            return;
        }
    } else if let Err(err) = conf.merge(config::File::with_name("conf.yml")) {
        error!("Unable to load config file: {}", err);
        return;
    }

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .with_context(|| handle_error("Expected env variable DISCORD_TOKEN to be set".to_string()))
        .unwrap();

    let http = Http::new_with_token(&conf.get_str("DISCORD_TOKEN").unwrap());

    // Fetch bot's owners and id
    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(bot_id) => (owners, bot_id.id),
                Err(err) => panic!("Could not access the bot id: {:?}", err),
            }
        }
        Err(err) => panic!("Could not access application info: {:?}", err),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("!")
                .delimiters(vec![", ", ","])
                .owners(owners)
        })
        .help(&HELP)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .intents(
            GatewayIntents::GUILD_VOICE_STATES
                | GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MESSAGES,
        )
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
    }
}
