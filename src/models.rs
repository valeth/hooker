pub mod discord;
pub mod gitlab;

use serde::{Serialize, Deserialize};
pub use uuid::Uuid as HookId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub id: HookId,
    pub description: String,
    pub gitlab_token: String,
    pub discord_url: String,
}

impl From<CreateHookConfig> for HookConfig {
    fn from(conf: CreateHookConfig) -> Self {
        let id = HookId::new_v4();

        Self {
            id,
            description: conf.description,
            gitlab_token: conf.gitlab_token,
            discord_url: conf.discord_url,
        }
    }

}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateHookConfig {
    pub description: String,
    pub gitlab_token: String,
    pub discord_url: String,
}
