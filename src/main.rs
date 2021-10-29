use std::{collections::HashSet, env};

use anyhow::Context as AnyhowCtx;
use log::{error, info};
use serenity::{
    client::bridge::gateway::GatewayIntents, framework::StandardFramework, http::Http, prelude::*,
};
use songbird::SerenityInit;

use crate::commands::GENERAL_GROUP;
use crate::events::handler::Handler;
use crate::{commands::help::HELP, utils::error::handle_error};

mod commands;
mod events;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Pascal starting...");

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .with_context(|| handle_error("Expected env variable DISCORD_TOKEN to be set".to_string()))
        .unwrap();

    let http = Http::new_with_token(&token);

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
