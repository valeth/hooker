pub mod discord;
pub mod gitlab;

use serde::Deserialize;
use crate::store::{HookConfig, HookId};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateHookConfig {
    pub description: String,
    pub gitlab_token: String,
    pub discord_url: String,
}

impl Into<HookConfig> for CreateHookConfig {
    fn into(self) -> HookConfig {
        HookConfig {
            id: HookId::new(),
            description: self.description,
            gitlab_token: self.gitlab_token,
            discord_url: self.discord_url,
            created_at: chrono::Utc::now(),
        }
    }

}
