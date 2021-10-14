use anyhow::{anyhow, Error};
use log::error;

pub fn handle_error(msg: String) -> Error {
    error!("{}", msg);

    return anyhow!(msg);
}
