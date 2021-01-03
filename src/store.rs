use std::{
    collections::HashMap,
    io::{self, Write},
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    convert::TryFrom,
    sync::Arc,
    fmt::{self, Display},
};
use anyhow::anyhow;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use crate::Result;

const STORAGE_ROOT: &str = "./data";

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct HookId(uuid::Uuid);

impl HookId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Display for HookId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl TryFrom<&str> for HookId {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        use std::str::FromStr;
        let uuid = uuid::Uuid::from_str(s)?;
        Ok(Self(uuid))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub id: HookId,
    pub description: String,
    pub gitlab_token: String,
    pub discord_url: String,
}

#[derive(Clone, Default)]
pub struct HookRegistry {
    inner: Arc<RwLock<HashMap<HookId, HookConfig>>>,
}

impl HookRegistry {
    pub fn load() -> Result<Self> {
        let config = load_all_hook_configs()?;
        Ok(Self {
            inner: Arc::new(RwLock::new(config))
        })
    }

    pub async fn all(&self) -> Vec<HookConfig> {
        let configs = self.inner.read().await;
        configs.values().cloned().collect()
    }

    pub async fn get<I>(&self, id: I) -> Result<HookConfig>
    where HookId: TryFrom<I>
    {
        let id = HookId::try_from(id)
            .or_else(|_| Err(anyhow!("Failed to parse id")))?;

        let configs = self.inner.read().await;
        let config = configs.get(&id)
            .ok_or_else(|| anyhow!("No hook config found for id"))?;

        Ok(config.clone())
    }

    pub async fn insert(&mut self, config: HookConfig) -> Result<()> {
        let mut configs = self.inner.write().await;
        let id = config.id.clone();
        store_hook_config(&config)?;
        configs.insert(id, config);

        Ok(())
    }

    pub async fn delete<I>(&mut self, id: I) -> Result<()>
    where HookId: TryFrom<I>
    {
        let id = HookId::try_from(id).or_else(|_| Err(anyhow!("Failed to parse id")))?;
        let mut configs = self.inner.write().await;
        delete_hook_config(&id)?;
        configs.remove(&id);

        Ok(())
    }
}

fn store_hook_config(config: &HookConfig) -> io::Result<()> {
    let json = serde_json::to_string_pretty(config)?;
    let outfile = get_hook_path()?.join(format!("{}.json", config.id));
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(outfile)?;

    file.write(json.as_bytes())?;

    Ok(())
}

fn delete_hook_config(id: &HookId) -> io::Result<()> {
    let path = get_hook_path()?.join(format!("{}.json", id));
    log::debug!("Deleting hook configuration at {}", path.display());
    fs::remove_file(path)?;
    Ok(())
}

fn load_hook_config<P: AsRef<Path>>(path: P) -> io::Result<HookConfig> {
    log::debug!("Loading hook configuration from {}", path.as_ref().display());
    let infile = File::open(path)?;
    Ok(serde_json::from_reader(infile)?)
}

fn load_all_hook_configs() -> io::Result<HashMap<HookId, HookConfig>> {
    let mut configs = HashMap::new();
    let path = get_hook_path()?;
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_file() {
            let config = load_hook_config(path)?;
            let id = config.id.clone();
            configs.insert(id, config);
        }
    }

    Ok(configs)
}

fn get_hook_path() -> io::Result<PathBuf> {
    let path = PathBuf::from(STORAGE_ROOT)
        .join("hooks");
    fs::create_dir_all(&path)?;
    Ok(path)
}
