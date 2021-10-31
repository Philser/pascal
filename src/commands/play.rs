use log::error;
use serenity::client::Context;

use anyhow::Context as AnyhowCtx;
use anyhow::Result;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use serenity::Result as SerenityResult;
use songbird::input;

use crate::utils::discord::{join_channel, play_sound};
use crate::utils::error::handle_error;
use crate::utils::sound_files::get_sound_files;

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

    let channel_id = match channel_id_opt {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    if arg.starts_with("https://") {
        play_youtube(ctx, msg, &guild, channel_id, &arg).await?;
    } else {
        play_from_file(ctx, msg, guild, channel_id, &arg).await?;
    }

    Ok(())
}

async fn play_from_file(
    ctx: &Context,
    msg: &Message,
    guild: Guild,
    channel_id: ChannelId,
    file_name: &str,
) -> Result<()> {
    let sound_files = get_sound_files()?;

    match sound_files.get(file_name) {
        Some(file) => {
            join_channel(ctx, guild.id, channel_id)
                .await
                .with_context(|| handle_error("Failed to join channel".to_string()))?;
            crate::utils::discord::play_sound(ctx, guild.id, file).await?;
        }
        None => {
            // TODO: Refactor into error methods or smth
            check_msg(
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "I don't know this sound: **{}**\nType `!list` to see a list of sounds",
                            file_name
                        ),
                    )
                    .await,
            );
        }
    }

    Ok(())
}

// TODO: Refactor like play_sound, i.e. more specialized and modular
async fn play_youtube(
    ctx: &Context,
    msg: &Message,
    guild: &Guild,
    channel_id: ChannelId,
    url: &str,
) -> Result<()> {
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| handle_error("Songbird client not initialized".to_string()))?;

    join_channel(ctx, guild.id, channel_id)
        .await
        .with_context(|| handle_error("Failed to join channel".to_string()))?;

    let handler_lock = manager
        .get(guild.id)
        .ok_or_else(|| handle_error("Couldn't get handler lock".to_string()))?;

    let mut handler = handler_lock.lock().await;

    match songbird::ytdl(&url).await {
        Ok(source) => handler.play_only_source(source),
        Err(err) => {
            let err_message = format!("Error streaming youtube source: {}", err);
            check_msg(msg.channel_id.say(&ctx.http, err_message.clone()).await);

            return Err(handle_error(err_message));
        }
    };

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs err to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(err) = result {
        error!("Error sending message: {:?}", err);
    }
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
