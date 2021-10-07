use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::guild::Guild;
use serenity::Result as SerenityResult;

use crate::SoundStore;

/// Plays available sounds.
/// Use the !list command to get a list of available sounds.
/// Usage: `!play [sound name]`
#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or("Message not received via gateway, but is required to be")?;
    let guild = msg.guild(&ctx.cache).await.unwrap();

    join_channel(ctx, msg, &guild).await.unwrap();

    let sound_name = parse_sound_name(&ctx, &msg, &mut args)
        .await
        .ok_or("Did not provide sound name")?;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let sources_lock = ctx
            .data
            .read()
            .await
            .get::<SoundStore>()
            .cloned()
            .expect("Sound cache was installed at startup.");

        let sources = sources_lock.lock().await;

        let source = sources.get(&sound_name).expect("Sound file missing");

        let _sound = handler.play_source(source.into());

        check_msg(msg.channel_id.say(&ctx.http, &sound_name).await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

async fn join_channel(ctx: &Context, msg: &Message, guild: &Guild) -> CommandResult {
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let join = manager.join(guild_id, connect_to).await;

    join.1?;

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs err to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(err) = result {
        println!("Error sending message: {:?}", err);
    }
}

/// Validates given sound name and responds to errors with a feedback message
async fn parse_sound_name(ctx: &Context, msg: &Message, args: &mut Args) -> Option<String> {
    let sound_name = match args.single::<String>() {
        Ok(sound_name) => sound_name,
        Err(_) => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Must provide name of the sound to play")
                    .await,
            );

            return None;
        }
    };

    Some(sound_name)
}
