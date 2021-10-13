use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
#[only_in(guilds)]
#[aliases(s)]
pub async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg
        .guild(&ctx.cache)
        .await
        .ok_or("No guild present in cache")?;

    let channel = msg
        .channel(&ctx.cache)
        .await
        .ok_or("Missing channel in cache")?;

    let guild_channel = channel.guild().ok_or("Channel not in guild")?;
    if !guild_channel.name().eq("pascal-phone") {
        return Ok(());
    }

    let manager = songbird::get(ctx)
        .await
        .ok_or("Songbird Voice client placed in at initialisation.")?
        .clone();

    let handler_lock = manager.get(guild.id).ok_or("Couldn't get handler lock")?;
    let mut handler = handler_lock.lock().await;

    handler.stop();

    Ok(())
}
