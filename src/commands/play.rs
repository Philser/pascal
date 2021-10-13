use serenity::client::Context;

use serenity::framework::standard::CommandError;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::guild::Guild;
use serenity::Result as SerenityResult;
use songbird::input;

use crate::utils::sound_files::get_sound_files;

#[command]
#[only_in(guilds)]
#[aliases(p)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .expect("No guild present in cache");

    let channel = msg
        .channel(&ctx.cache)
        .await
        .expect("Missing channel in cache");

    let guild_channel = channel.guild().expect("Channel not in guild");
    if !guild_channel.name().eq("pascal-phone") {
        return Ok(());
    }

    let arg = parse_argument(ctx, msg, &mut args).await.unwrap();

    if arg.starts_with("https://") {
        return play_youtube(ctx, msg, &guild, &arg).await;
    } else {
        return play_sound(ctx, msg, &guild, &arg).await;
    }
}

async fn play_youtube(ctx: &Context, msg: &Message, guild: &Guild, url: &str) -> CommandResult {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    join_channel(ctx, msg, &guild).await.unwrap();

    let handler_lock = manager.get(guild.id).expect("Couldn't get handler lock");
    let mut handler = handler_lock.lock().await;

    let source = match songbird::ytdl(&url).await {
        Ok(source) => source,
        Err(err) => {
            check_msg(
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!("Error streaming youtube source: {}", err),
                    )
                    .await,
            );

            return Ok(());
        }
    };

    handler.play_only_source(source);

    Ok(())
}

async fn play_sound(
    ctx: &Context,
    msg: &Message,
    guild: &Guild,
    sound_name: &str,
) -> CommandResult {
    let sound_files = get_sound_files().map_err(|err| format!("{}", err)).unwrap();

    let file = sound_files.get(sound_name);

    if let Some(sound_file) = file {
        let src = input::ffmpeg(sound_file.file_path.clone()).await.unwrap();

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        join_channel(ctx, msg, &guild).await.unwrap();

        let handler_lock = manager.get(guild.id).expect("Couldn't get handler lock");
        let mut handler = handler_lock.lock().await;

        handler.play_source(src.into());
    } else {
        // TODO: Refactor into error methods or smth
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
        .expect("Error fetching Songbird client")
        .clone();

    let join = manager.join(guild_id, connect_to).await;

    join.1.unwrap();

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs err to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(err) = result {
        println!("Error sending message: {:?}", err);
    }
}

/// Validates given sound name and responds to errors with a feedback message
async fn parse_argument(
    ctx: &Context,
    msg: &Message,
    args: &mut Args,
) -> Result<String, CommandError> {
    let sound_name = match args.single::<String>() {
        Ok(sound_name) => sound_name,
        Err(_) => {
            let err = "Must provide name of the sound to play or a Youtube URL";
            check_msg(
                msg.channel_id
                    .say(
                        &ctx.http,
                        "Must provide name of the sound to play or a Youtube URL",
                    )
                    .await,
            );

            return Err(Box::from(err));
        }
    };

    Ok(sound_name)
}
