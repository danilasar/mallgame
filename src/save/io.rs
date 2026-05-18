use std::fs;
use std::io::Write;
use std::path::Path;
use crate::save::types::SaveGame;

pub fn write_save_file_atomic(path: impl AsRef<Path>, save: &SaveGame) -> Result<(), String> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(save).map_err(|e| format!("Failed to serialize: {}", e))?;
    
    let tmp_path = path.with_extension("tmp");
    
    let mut file = fs::File::create(&tmp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(json.as_bytes()).map_err(|e| format!("Failed to write to temp file: {}", e))?;
    file.flush().map_err(|e| format!("Failed to flush: {}", e))?;
    
    fs::rename(&tmp_path, path).map_err(|e| format!("Failed to rename temp file to save file: {}", e))?;
    
    Ok(())
}

pub fn read_save_file(path: impl AsRef<Path>) -> Result<SaveGame, String> {
    let path = path.as_ref();
    let json = fs::read_to_string(path).map_err(|e| format!("Failed to read save file: {}", e))?;
    let save: SaveGame = serde_json::from_str(&json).map_err(|e| format!("Failed to parse save file: {}", e))?;
    Ok(save)
}
