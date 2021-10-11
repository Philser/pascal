use std::{collections::HashMap, error::Error, fs, path::PathBuf};

const ALLOWED_TYPES: [&str; 3] = ["m4a", "wav", "mp3"];

pub struct SoundFile {
    pub file_name: String,
    pub file_extension: String,
    pub file_path: PathBuf,
}

// TODO: Refactor
/// Crawls the designated sound file directory for all allowed file extensions
/// and returns a mapping of sound name (sound file minus extension) to sound file information
pub fn get_sound_files() -> Result<HashMap<String, SoundFile>, Box<dyn Error>> {
    let mut sound_files: HashMap<String, SoundFile> = HashMap::new();
    if let Ok(files) = fs::read_dir("./audio") {
        for file in files {
            // filter for allowed extensions
            let dir_entry = file.as_ref().unwrap();
            let path = dir_entry.path();
            if let Some(extension) = path.extension() {
                if let Some(ext) = extension.to_str() {
                    if ALLOWED_TYPES.contains(&ext) {
                        // check if file name is the desired one
                        let filename = dir_entry.file_name();
                        if let Some(raw_name) = filename.to_str() {
                            sound_files.insert(
                                raw_name.replace(&format!(".{}", ext), ""),
                                SoundFile {
                                    file_name: raw_name.to_owned(),
                                    file_extension: ext.to_owned(),
                                    file_path: path.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(sound_files)
}
