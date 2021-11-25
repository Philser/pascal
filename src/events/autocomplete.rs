use log::error;
use serde_json::Value;
use serenity::{client::Context, model::interactions::autocomplete::AutocompleteInteraction};

use crate::utils::fuzzy_lookup;

use super::slash_commands::PLAY_COMMAND;

pub async fn handle_autocomplete_interaction(ctx: Context, autocomplete: AutocompleteInteraction) {
    if autocomplete.data.name == PLAY_COMMAND {
        let searched_sound_opt = match autocomplete
            .data
            .options
            .iter()
            .filter(|option| option.name == "sound")
            .last()
        {
            Some(s) => s.value.clone(),
            None => return, // Ignore unknown command names
        };

        let searched_sound_value = match searched_sound_opt {
            Some(s) => s,
            None => Value::String("".to_string()),
        };

        let searched_sound = match searched_sound_value.as_str() {
            Some(s) => s,
            None => return, // Whatever the hell arrives here, we don't want anything to do with it
        };

        let sound_files: Vec<String> = match crate::utils::sound_files::get_sound_files() {
            Ok(map) => map.keys().map(|key| key.to_owned()).collect(),
            Err(e) => {
                error!(
                    "[Autocomplete Interaction] Error fetching sound files: {}",
                    e
                );
                return;
            }
        };

        let suggestions = fuzzy_lookup::get_lookup_results(searched_sound, sound_files);

        if let Err(e) = autocomplete
            .create_autocomplete_response(&ctx.http, |response| {
                for suggestion in suggestions {
                    response.add_string_choice(suggestion.clone(), suggestion);
                }

                response
            })
            .await
        {
            error!("Error sending auto complete suggestions: {}", e);
        }
    }
}
