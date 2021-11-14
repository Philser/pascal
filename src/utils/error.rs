use anyhow::{anyhow, Error};
use log::error;
use serenity::model::channel::Message;
use serenity::Result as SerenityResult;

pub fn handle_error(msg: String) -> Error {
    error!("{}", msg);

    return anyhow!(msg);
}

/// Checks that a message successfully sent; if not, then logs err to stdout.
pub fn check_msg(result: SerenityResult<Message>) {
    if let Err(err) = result {
        error!("Error sending message: {:?}", err);
    }
}
