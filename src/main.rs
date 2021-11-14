use std::fs;
use std::sync::Arc;
use std::{collections::HashSet, env};

use anyhow::Result;
use log::{error, info};
use serenity::{
    client::bridge::gateway::GatewayIntents, framework::StandardFramework, http::Http, prelude::*,
};
use songbird::SerenityInit;
use utils::config::IntroConfig;

use crate::commands::help::HELP;
use crate::commands::GENERAL_GROUP;
use crate::events::handler::Handler;
use crate::utils::config::Config;

mod commands;
mod events;
mod utils;

struct IntroStore;

impl TypeMapKey for IntroStore {
    type Value = Arc<Mutex<IntroConfig>>;
}

#[tokio::main]
async fn main() {
    env_logger::init();

    info!("Pascal starting...");

    let conf = match load_conf() {
        Ok(conf) => conf,
        Err(err) => {
            error!("Unable to load config file: {}", err);
            return;
        }
    };

    let http = Http::new_with_token(&conf.discord_token);

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

    let mut client = Client::builder(&conf.discord_token)
        .framework(framework)
        .event_handler(Handler)
        .application_id(conf.application_id)
        .intents(
            GatewayIntents::GUILD_VOICE_STATES
                | GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MESSAGES,
        )
        .register_songbird()
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;

        data.insert::<IntroStore>(Arc::new(Mutex::new(conf.intros)));
    }

    if let Err(err) = client.start().await {
        error!("Client error: {:?}", err);
    }
}

fn load_conf() -> Result<Config> {
    let mut conf_file: String = "config.yml".to_owned();

    // If no custom conf file is specified, look for conf.yml
    if let Ok(alt_conf_file) = env::var("CONFIG") {
        conf_file = alt_conf_file;
    }

    let conf: Config = serde_yaml::from_reader(fs::File::open(conf_file)?)?;

    Ok(conf)
}
