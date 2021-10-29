use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    pub discord_token: String,
    pub intros: IntroConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IntroConfig {
    pub channels: Vec<u64>,
    pub user_intros: Vec<UserIntro>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserIntro {
    pub user: u64,
    pub sound_file: String,
}
