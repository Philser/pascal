use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::guild::Guild;
use serenity::Result as SerenityResult;
use songbird::input;

use crate::utils::sound_files::get_sound_files;

#[command]
#[only_in(guilds)]
#[aliases(p)]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(channel) = msg.channel(&ctx.cache).await {
        if let Some(guild_channel) = channel.guild() {
            if guild_channel.name().eq("pascal-phone") {
                return play_sound(ctx, msg, args).await;
            }
        }
    }

    Ok(())
}

async fn play_sound(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg
        .guild_id
        .ok_or("Message not received via gateway, but is required to be")?;
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let sound_name = parse_sound_name(&ctx, &msg, &mut args)
        .await
        .ok_or("Did not provide sound name")?;

    let sound_files = get_sound_files().map_err(|err| format!("{}", err))?;

    let file = sound_files.get(&sound_name);

    if file.is_none() {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!(
                        "I don't know this sound: **{}**\nType `!list` to see a list of sounds",
                        sound_name
                    ),
                )
                .await,
        );

        return Ok(());
    } else {
        let src = input::ffmpeg(file.unwrap().file_path.clone()).await?;

        join_channel(ctx, msg, &guild).await.unwrap();

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        if let Some(handler_lock) = manager.get(guild_id) {
            let mut handler = handler_lock.lock().await;

            // For now, we don't want to queue sounds, because that might get messy quickly.
            if handler.queue().len() > 0 {
                return Ok(());
            }

            handler.enqueue_source(src.into());
        }
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
