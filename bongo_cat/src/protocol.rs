use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct IpcEvent {
    pub action: String,
    pub json: String,
}
