use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub open_file: Option<PathBuf>,
    pub last_folder: Option<PathBuf>,
}
