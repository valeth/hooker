use std::{
    collections::HashMap,
    io::{self, Write},
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};
use crate::models::{HookId, HookConfig};

const STORAGE_ROOT: &str = "./data";

pub fn store_hook_config(config: &HookConfig) -> io::Result<()> {
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

pub fn delete_hook_config(id: &HookId) -> io::Result<()> {
    let path = get_hook_path()?.join(format!("{}.json", id));
    log::debug!("Deleting hook configuration at {}", path.display());
    fs::remove_file(path)?;
    Ok(())
}

pub fn load_hook_config<P: AsRef<Path>>(path: P) -> io::Result<HookConfig> {
    log::debug!("Loading hook configuration from {}", path.as_ref().display());
    let infile = File::open(path)?;
    Ok(serde_json::from_reader(infile)?)
}

pub fn load_all_hook_configs() -> io::Result<HashMap<HookId, HookConfig>> {
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
