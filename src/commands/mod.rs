use crate::commands::list::LIST_COMMAND;
use crate::commands::play::PLAY_COMMAND;
use crate::commands::stop::STOP_COMMAND;
use serenity::framework::standard::macros::group;

pub mod help;
pub mod list;
pub mod play;
pub mod stop;

#[group]
#[commands(play, list, stop)]
struct General;
