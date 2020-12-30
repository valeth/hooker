use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookConfig {
    pub id: String,
    pub description: String,
    pub gitlab_token: String,
    pub discord_url: String,
}
