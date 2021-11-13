use serenity::client::Context;

use anyhow::Context as AnyhowCtx;
use anyhow::Result;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use songbird::input;

use crate::utils::error::check_msg;
use crate::utils::error::handle_error;

use super::sound_files::get_sound_files;
use super::sound_files::SoundFile;

pub async fn play_sound(ctx: &Context, guild_id: GuildId, sound_file: &SoundFile) -> Result<()> {
    let src = input::ffmpeg(sound_file.file_path.clone())
        .await
        .with_context(|| handle_error("Error reading ffmpeg source".to_string()))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| handle_error("Songbird Voice client not initialized".to_string()))?
        .clone();

    let handler_lock = manager
        .get(guild_id)
        .ok_or_else(|| handle_error("Couldn't get handler lock".to_string()))?;

    let mut handler = handler_lock.lock().await;
    handler.play_source(src);

    Ok(())
}

pub async fn play_from_file(
    ctx: &Context,
    channel_id: ChannelId,
    guild_id: GuildId,
    file_name: &str,
) -> Result<()> {
    let sound_files = get_sound_files()?;

    match sound_files.get(file_name) {
        Some(file) => {
            crate::utils::discord::play_sound(ctx, guild_id, file).await?;
        }
        None => {
            // TODO: Refactor into error methods or smth
            check_msg(
                channel_id
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
pub async fn play_youtube(
    ctx: &Context,
    channel_id: ChannelId,
    guild_id: GuildId,
    url: &str,
) -> Result<()> {
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| handle_error("Songbird client not initialized".to_string()))?;

    let handler_lock = manager
        .get(guild_id)
        .ok_or_else(|| handle_error("Couldn't get handler lock".to_string()))?;

    let mut handler = handler_lock.lock().await;

    match songbird::ytdl(&url).await {
        Ok(source) => handler.play_only_source(source),
        Err(err) => {
            let err_message = format!("Error streaming youtube source: {}", err);
            check_msg(channel_id.say(&ctx.http, err_message.clone()).await);

            return Err(handle_error(err_message));
        }
    };

    Ok(())
}

pub async fn join_channel(ctx: &Context, guild_id: GuildId, channel_id: ChannelId) -> Result<()> {
    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| handle_error("Error fetching Songbird client".to_string()))?
        .clone();

    let join = manager.join(guild_id, channel_id).await;

    join.1?;

    Ok(())
}
