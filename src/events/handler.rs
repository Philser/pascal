use anyhow::Context as AnyhowCtx;
use log::{debug, error, info};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction,
        },
        prelude::*,
    },
};

use crate::utils::{
    discord::{get_channel_of_member, join_channel, play_from_file, play_sound, play_youtube},
    error::{check_msg, handle_error},
    sound_files::get_sound_files,
};
use crate::{utils::config::UserIntro, IntroStore};

pub(crate) struct Handler;

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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let guild_id = match command.guild_id {
                Some(gid) => gid,
                None => {
                    debug!(
                        "Guild ID not found for slash command {} and caller {}",
                        command.data.name, command.user.name
                    );
                    return;
                }
            };

            if command.data.name.as_str() == "play" {
                if let Some(channel_id) =
                    get_channel_of_member(ctx.clone(), guild_id, command.user.id).await
                {
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
                            .with_context(|| {
                                handle_error("Failed to play from youtube".to_string())
                            })
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
                            .interaction_response_data(|message| {
                                message.content("Tight.").flags(flags)
                            })
                    })
                    .await
                {
                    error!("Error responding to slash command: {}", e);
                }
            };
        } else if let Interaction::Autocomplete(autocomplete) = interaction {
            if let Err(e) = autocomplete
                .create_autocomplete_response(&ctx.http, |response| {
                    response
                        .add_string_choice("test", "test")
                        .add_string_choice("test2", "test2")
                })
                .await
            {
                error!("Error sending auto complete suggestions: {}", e);
            }
        }
    }

    // Detects when a known user joins an allowed channel and plays a registered intro song
    // TODO: Refactor into own method
    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
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

        // Only play intros for users that weren't on the server before (i.e. in another channel)
        if old_state.is_some() {
            return;
        }

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

        if guild_id.is_none() {
            debug!("No guild ID present, cannot play intro song");
            return;
        }
        let guild_id = guild_id.unwrap();

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
            debug!("Error joining voice channel: {}", err);
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
                    return;
                }
            }
            Err(err) => {
                error!("{}", err);
                return;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        match ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| {
                command
                    .name("play")
                    .description("Command Pascal to play a sound")
                    .create_option(|option| {
                        option
                            .name("sound")
                            .description("Name of the sound to play. Use the list command to see all possible values")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                            .set_autocomplete(true)
                    })
            })
        })
        .await
        {
            Ok(_) => (),
            Err(e) => error!("Error registering slash commands: {}", e),
        }

        println!("{} is connected!", ready.user.name);
    }
}
