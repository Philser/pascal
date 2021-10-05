use crate::commands::play::PLAY_COMMAND;
use serenity::framework::standard::macros::group;

pub mod play;

#[group]
#[commands(play)]
struct General;
