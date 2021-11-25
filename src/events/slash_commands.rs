use std::fmt;

use anyhow::Context as AnyhowCtx;
use log::error;
use serenity::{
    client::Context,
    model::{
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
            },
            InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
        },
    },
};

use crate::utils::{
    discord::{get_channel_of_member, join_channel, play_from_file, play_youtube},
    error::{check_msg, handle_error},
};

pub const PLAY_COMMAND: &str = "play";

pub async fn handle_slash_commands(ctx: Context, command: ApplicationCommandInteraction) {
    let guild_id = match command.guild_id {
        Some(gid) => gid,
        None => {
            error!(
                "Guild ID not found for slash command {} and caller {}",
                command.data.name, command.user.name
            );
            return;
        }
    };

    if command.data.name.as_str() == PLAY_COMMAND {
        handle_play_command(ctx, command, guild_id).await
    };
}

async fn handle_play_command(
    ctx: Context,
    command: ApplicationCommandInteraction,
    guild_id: GuildId,
) {
    if let Some(channel_id) = get_channel_of_member(ctx.clone(), guild_id, command.user.id).await {
        if let Err(e) = join_channel(&ctx, guild_id, channel_id).await {
            error!("Failed to join channel: {}", e);
        }
    }

    // Only use the first provided argument
    let option = command
        .data
        .options
        .get(0)
        .expect("Expected name of sound to play")
        .resolved
        .as_ref()
        .expect("Expected resolved command option");

    if let ApplicationCommandInteractionDataOptionValue::String(sound_name) = option {
        if sound_name.starts_with("https://") {
            play_youtube(&ctx, command.channel_id, guild_id, sound_name)
                .await
                .with_context(|| handle_error("Failed to play from youtube".to_string()))
                .unwrap();
        } else {
            play_from_file(&ctx, command.channel_id, guild_id, sound_name)
                .await
                .with_context(|| handle_error("Failed to play sound".to_string()))
                .unwrap();
        }
    } else {
        check_msg(
            command
                .channel_id
                .say(&ctx.http, "Invalid sound name input")
                .await,
        );
    }

    let flags = InteractionApplicationCommandCallbackDataFlags::EPHEMERAL;
    if let Err(e) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Tight.").flags(flags))
        })
        .await
    {
        error!("Error responding to slash command: {}", e);
    }
}
