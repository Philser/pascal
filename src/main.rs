use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    error::Error,
    sync::Arc,
};

use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    framework::StandardFramework,
    http::Http,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use songbird::{
    input::{
        self,
        cached::{Compressed, Memory},
        Input,
    },
    SerenityInit,
};
use std::fs;

use crate::commands::help::HELP;
use crate::commands::GENERAL_GROUP;

mod commands;

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

struct SoundStore;

impl TypeMapKey for SoundStore {
    type Value = Arc<Mutex<HashMap<String, CachedSound>>>;
}

enum CachedSound {
    Compressed(Compressed),
    Uncompressed(Memory),
}

impl From<&CachedSound> for Input {
    fn from(obj: &CachedSound) -> Self {
        use CachedSound::*;
        match obj {
            Compressed(c) => c.new_handle().into(),
            Uncompressed(u) => u
                .new_handle()
                .try_into()
                .expect("Failed to create decoder for Memory source."),
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    dotenv::dotenv().expect("Missing .env file");
    let token = dotenv::var("DISCORD_TOKEN").expect("Expected entry DISCORD_TOKEN in .env");

    let http = Http::new_with_token(&token);
    // http.get_guild(guild_id)

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

    let sounds = load_sounds()
        .await
        .map_err(|err| format!("Error loading sounds: {}", err))
        .unwrap();

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

    {
        let mut client_data = client.data.write().await;
        client_data.insert::<SoundStore>(Arc::new(Mutex::new(sounds)));
    }

    if let Err(err) = client.start().await {
        println!("Client error: {:?}", err);
    }
}

async fn load_sounds() -> Result<HashMap<String, CachedSound>, Box<dyn Error>> {
    // Loading the audio ahead of time.
    let mut audio_map: HashMap<String, CachedSound> = HashMap::new();

    let allowed_types = vec!["m4a", "wav"];

    let files = fs::read_dir("./audio")?;
    for file in files {
        let dir_entry = file?;
        let path = dir_entry.path();
        let filename = dir_entry.file_name();
        if let Some(raw_name) = filename.to_str() {
            if let Some(extension) = path.extension() {
                if let Some(ext) = extension.to_str() {
                    if allowed_types.contains(&ext) {
                        let src =
                            Memory::new(input::ffmpeg(path.clone()).await?).map_err(|err| {
                                format!("Error creating memory sound object: {}", err)
                            })?;

                        src.raw.spawn_loader();

                        audio_map.insert(
                            raw_name.replace(&format!(".{}", ext), ""),
                            CachedSound::Uncompressed(src),
                        );
                    }
                }
            }
        }
    }

    Ok(audio_map)
}
