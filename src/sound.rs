use app_dirs::{self, AppInfo, AppDataType};

use std::fs::{self, File};
use std::io::Write;

const APP_INFO: AppInfo = AppInfo {
    name: "chip8_bin",
    author: "pengowen",
};

/// Data for the beep sound used by the emulator
const BEEP_SOUND: &'static [u8] = include_bytes!("../beep.wav");

/// Returns the path to the sound file
/// Creates the file and writes the sound data to it if the file doesn't exist
pub fn sound_path() -> String {
    // Get the path
    let path = app_dirs::app_root(AppDataType::UserData, &APP_INFO)
        .unwrap_or_else(|e| panic!("Failed to get app data directory: {}", e))
        .join("beep.wav")
        .to_str()
        .unwrap_or_else(|| panic!("Path to sound file was invalid"))
        .to_string();

    // Test if the file exists (fs::metadata returns an error if it doesn't)
    // If the file does exist, but fs::metadata returns an error anyways, the error will be caught
    // when attempting to create the file
    if fs::metadata(&path).is_err() {
        // Create the file and write the data to it
        File::create(&path)
            .and_then(|mut f| f.write_all(BEEP_SOUND))
            .unwrap_or_else(|e| panic!("Failed to create sound file: {}", e));
    }

    path
}
