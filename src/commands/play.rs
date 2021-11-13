use serenity::client::Context;

use anyhow::Context as AnyhowCtx;
use anyhow::Result;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;

use crate::utils::discord::join_channel;
use crate::utils::discord::play_from_file;
use crate::utils::discord::play_youtube;
use crate::utils::error::check_msg;
use crate::utils::error::handle_error;

#[command]
#[only_in(guilds)]
#[aliases(p)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| handle_error("No guild present in cache".to_string()))?;

    let channel = msg
        .channel(&ctx.cache)
        .await
        .ok_or_else(|| handle_error("Missing channel in cache".to_string()))?;

    let guild_channel = channel
        .guild()
        .ok_or_else(|| handle_error("Channel not in guild".to_string()))?;
    if !guild_channel.name().eq("pascal-phone") {
        return Ok(());
    }

    let arg = parse_argument(ctx, msg, &mut args).await?;

    let channel_id_opt = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    // It's desired behaviour to have the bot play the sound in the current channel
    // if the caller is not in a channel themself
    match channel_id_opt {
        Some(channel) => join_channel(ctx, guild.id, channel)
            .await
            .with_context(|| handle_error("Failed to join channel".to_string()))?,
        None => check_msg(msg.reply(ctx, "Not in a voice channel").await),
    };

    if arg.starts_with("https://") {
        play_youtube(ctx, msg.channel_id, guild.id, &arg).await?;
    } else {
        play_from_file(ctx, msg.channel_id, guild.id, &arg).await?;
    }

    Ok(())
}

/// Validates given sound name and responds to errors with a feedback message
async fn parse_argument(ctx: &Context, msg: &Message, args: &mut Args) -> Result<String> {
    let sound_name = match args.single::<String>() {
        Ok(sound_name) => sound_name,
        Err(_) => {
            let err = "Must provide name of the sound to play or a Youtube URL";
            check_msg(msg.channel_id.say(&ctx.http, err).await);

            return Err(handle_error(err.to_string()));
        }
    };

    Ok(sound_name)
}
