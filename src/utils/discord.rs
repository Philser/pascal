use log::error;
use serenity::client::Context;

use anyhow::Context as AnyhowCtx;
use anyhow::Result;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use songbird::input;

use crate::utils::error::handle_error;

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
    // } else {

    // }

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
