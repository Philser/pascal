use super::{
    autocomplete::handle_autocomplete_interaction, slash_commands::handle_slash_commands,
    voice::handle_voice_state_update,
};
use log::error;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        id::GuildId,
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType},
            Interaction,
        },
        prelude::*,
    },
};

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
            handle_slash_commands(ctx, command).await;
        } else if let Interaction::Autocomplete(autocomplete) = interaction {
            handle_autocomplete_interaction(ctx, autocomplete).await;
        }
    }

    // Detects when a known user joins an allowed channel and plays a registered intro song
    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        handle_voice_state_update(ctx, guild_id, old_state, new_state).await;
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
