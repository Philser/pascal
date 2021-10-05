use serenity::client::Context;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::Result as SerenityResult;

/// Command responsible for playing sounds
#[command]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    join_channel(ctx, msg).await.unwrap();
    Ok(())
}

async fn join_channel(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = msg.guild_id.ok_or("Not received via gateway")?;

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

    let _ = manager.join(guild_id, connect_to).await;

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs err to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(err) = result {
        println!("Error sending message: {:?}", err);
    }
}
