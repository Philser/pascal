use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::utils::error::handle_error;

#[command]
#[only_in(guilds)]
#[aliases(s)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or_else(|| handle_error("No guild present in cache".to_string()))?;

    let channel = msg
        .channel(&ctx.cache)
        .await
        .ok_or_else(|| handle_error("Missing channel in cache".to_string()))?;

    let guild_channel = channel
        .guild()
        .ok_or_else(|| handle_error("Channel not in guild".to_string()))?;
    if !guild_channel.name().eq("pascal-phone") {
        return Ok(());
    }

    let manager = songbird::get(ctx)
        .await
        .ok_or_else(|| {
            handle_error("Songbird Voice client placed in at initialisation.".to_string())
        })?
        .clone();

    let handler_lock = manager
        .get(guild.id)
        .ok_or_else(|| handle_error("Couldn't get handler lock".to_string()))?;
    let mut handler = handler_lock.lock().await;

    handler.stop();

    Ok(())
}
