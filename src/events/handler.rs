use std::ops::RangeBounds;

use log::{error, info};
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{
        channel::Message,
        id::GuildId,
        prelude::{Ready, VoiceState},
    },
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

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild_id: Option<GuildId>,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        info!("Event received");
        // Only play intros for users that weren't on the server before (i.e. in another channel)
        if old_state.is_some() {
            return;
        }

        let intros_lock = ctx
            .data
            .read()
            .await
            .get::<IntroStore>()
            .cloned()
            .ok_or("[Voice State Update] Unable to get intro store")
            .map_err(|err| error!("{}", err))
            .unwrap();

        let intros = intros_lock.lock().await;

        // Only allowed channels
        if new_state.channel_id.is_none()
            || !intros
                .channels
                .contains(&new_state.channel_id.unwrap().as_u64())
        {
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

        let intro_file: &str = intros
            .user_intros
            .iter()
            .filter(|user_intro| user_intro.user == *new_state.user_id.as_u64())
            .collect::<Vec<&UserIntro>>()[0]
            .sound_file
            .as_ref();

        info!("I got so faaaar");
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
