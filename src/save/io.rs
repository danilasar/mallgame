use crate::save::types::SaveGame;
use std::error::Error;
use std::fs;
use std::fmt;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub enum SaveIoError {
    Serialize(serde_json::Error),
    CreateTempFile(std::io::Error),
    WriteTempFile(std::io::Error),
    FlushTempFile(std::io::Error),
    RenameTempFile(std::io::Error),
    ReadSaveFile(std::io::Error),
    ParseSaveFile(serde_json::Error),
}

impl fmt::Display for SaveIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveIoError::Serialize(e) => write!(f, "Failed to serialize save file: {}", e),
            SaveIoError::CreateTempFile(e) => write!(f, "Failed to create temp file: {}", e),
            SaveIoError::WriteTempFile(e) => write!(f, "Failed to write to temp file: {}", e),
            SaveIoError::FlushTempFile(e) => write!(f, "Failed to flush temp file: {}", e),
            SaveIoError::RenameTempFile(e) => {
                write!(f, "Failed to rename temp file to save file: {}", e)
            }
            SaveIoError::ReadSaveFile(e) => write!(f, "Failed to read save file: {}", e),
            SaveIoError::ParseSaveFile(e) => write!(f, "Failed to parse save file: {}", e),
        }
    }
}

impl Error for SaveIoError {}

pub fn write_save_file_atomic(
    path: impl AsRef<Path>,
    save: &SaveGame,
) -> Result<(), SaveIoError> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(save).map_err(SaveIoError::Serialize)?;

    let tmp_path = path.with_extension("tmp");

    let mut file = fs::File::create(&tmp_path).map_err(SaveIoError::CreateTempFile)?;
    file.write_all(json.as_bytes())
        .map_err(SaveIoError::WriteTempFile)?;
    file.flush().map_err(SaveIoError::FlushTempFile)?;

    fs::rename(&tmp_path, path).map_err(SaveIoError::RenameTempFile)?;

    Ok(())
}

pub fn read_save_file(path: impl AsRef<Path>) -> Result<SaveGame, SaveIoError> {
    let path = path.as_ref();
    let json = fs::read_to_string(path).map_err(SaveIoError::ReadSaveFile)?;
    let save: SaveGame = serde_json::from_str(&json).map_err(SaveIoError::ParseSaveFile)?;
    Ok(save)
}
