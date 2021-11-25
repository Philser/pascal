use log::error;
use serenity::{
    client::Context,
    model::{id::GuildId, prelude::VoiceState},
};

use crate::{
    utils::{
        config::UserIntro,
        discord::{join_channel, play_sound},
        sound_files::get_sound_files,
    },
    IntroStore,
};

pub async fn handle_voice_state_update(
    ctx: Context,
    guild_id_opt: Option<GuildId>,
    old_state: Option<VoiceState>,
    new_state: VoiceState,
) {
    let guild_id = match guild_id_opt {
        Some(gid) => gid,
        None => {
            error!("No guild ID present, cannot play intro song");
            return;
        }
    };

    if old_state.is_none() && new_state.channel_id.is_some() {
        handle_voice_channel_intro(ctx, guild_id, new_state).await;
    }
}

async fn handle_voice_channel_intro(ctx: Context, guild_id: GuildId, new_state: VoiceState) {
    let intros_lock = match ctx
        .data
        .read()
        .await
        .get::<IntroStore>()
        .cloned()
        .ok_or("[Voice State Update] Unable to get intro store")
    {
        Ok(lock) => lock,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };
    let intros = intros_lock.lock().await;

    // Only allowed members
    if !intros
        .user_intros
        .iter()
        .map(|user_intro| user_intro.user)
        .any(|x| x == *new_state.user_id.as_u64())
    {
        return;
    }

    // Only care for updates in allowed channels
    if new_state.channel_id.is_none()
        || !intros
            .channels
            .contains(&new_state.channel_id.unwrap().as_u64())
    {
        return;
    }

    let intro_file: &str = intros
        .user_intros
        .iter()
        .filter(|user_intro| user_intro.user == *new_state.user_id.as_u64())
        .collect::<Vec<&UserIntro>>()[0]
        .sound_file
        .as_ref();

    let channel_id = match &new_state.channel_id {
        Some(cid) => cid,
        None => {
            error!("ChannelId was not present, cannot play intro song");
            return;
        }
    };

    if let Err(err) = join_channel(&ctx, guild_id, *channel_id).await {
        error!("Error joining voice channel: {}", err);
    }

    let sound_files = match get_sound_files() {
        Ok(files) => files,
        Err(err) => {
            error!("Error getting files: {}", err);
            return;
        }
    };

    match sound_files.get(intro_file).ok_or(format!(
        "Could not play intro file: Missing file {}",
        intro_file
    )) {
        Ok(file) => {
            if let Err(err) = play_sound(&ctx, guild_id, file).await {
                error!("Error playing sound: {}", err);
            }
        }
        Err(err) => {
            error!("{}", err);
        }
    }
}
