use crate::commands::list::LIST_COMMAND;
use crate::commands::play::PLAY_COMMAND;
use serenity::framework::standard::macros::group;

pub mod help;
pub mod list;
pub mod play;

#[group]
#[commands(play, list)]
struct General;
