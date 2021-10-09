use std::fs;

use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::{utils::sound_files::get_sound_files, SoundStore};

/// Lists all available sounds to play.
/// Usage: `!list'
#[command]
#[only_in(guilds)]
pub async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let files = get_sound_files().map_err(|err| format!("{}", err))?;

    let mut output = String::from("Type !play [sound name] to play a sound.\nAvailable sounds: \n");

    for sound_name in files.keys() {
        output.push_str(&format!("\t- {}\n", sound_name));
    }

    if let Err(why) = msg.channel_id.say(&ctx.http, output).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}
