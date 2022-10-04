use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const DB_CONFIG_DIR: &str = "d-rs";
const DB_FILENAME: &str = "dic.db";

pub static DB: Lazy<sled::Db> = Lazy::new(|| {
    let db_dir = dirs::data_local_dir().unwrap().join(DB_CONFIG_DIR);
    if db_dir.exists() && !db_dir.is_dir() {
        panic!("{} is not a directory", db_dir.display());
    } else if !db_dir.exists() {
        std::fs::create_dir_all(&db_dir).unwrap();
    }
    let db_path = db_dir.join(DB_FILENAME);
    let db = sled::open(&db_path).unwrap();
    db
});

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct History {
    pub word: String,
    pub last_searched: u64,
    pub last_reviewed: Option<u64>,
    pub searched_count: u64,
}

pub fn save_history(word: &str) -> anyhow::Result<()> {
    let history_tree = DB.open_tree("history")?;
    history_tree.update_and_fetch(word.as_bytes(), |old| {
        let mut history = if let Some(old) = old {
            serde_json::from_slice(old).unwrap_or_default()
        } else {
            History {
                word: word.to_string(),
                ..Default::default()
            }
        };
        history.last_searched = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        history.searched_count += 1;
        serde_json::to_vec(&history).ok()
    })?;

    DB.flush()?;

    Ok(())
}

pub fn remove_history(word: &str) -> anyhow::Result<()> {
    let history_tree = DB.open_tree("history")?;
    history_tree.remove(word.as_bytes())?;

    DB.flush()?;

    Ok(())
}
