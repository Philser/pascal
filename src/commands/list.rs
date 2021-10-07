use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::SoundStore;

/// Lists all available sounds to play.
/// Usage: `!list'
#[command]
#[only_in(guilds)]
pub async fn list(ctx: &Context, msg: &Message) -> CommandResult {
    let sound_files_lock = ctx
        .data
        .read()
        .await
        .get::<SoundStore>()
        .cloned()
        .expect("Sound cache was installed at startup.");

    let sound_files = sound_files_lock.lock().await;

    let mut output = String::from("Type !play [sound name] to play a sound.\nAvailable sounds: \n");

    for sound_name in sound_files.keys() {
        output.push_str(&format!("\t- {}\n", sound_name));
    }

    if let Err(why) = msg.channel_id.say(&ctx.http, output).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}
