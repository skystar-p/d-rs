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
    pub first_searched: i64,
    pub last_reviewed: Option<i64>,
    pub searched_count: i64,
}

pub fn save_history(word: &str, review: bool) -> anyhow::Result<()> {
    let history_tree = DB.open_tree("history")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    history_tree.update_and_fetch(word.as_bytes(), |old| {
        let mut history = if let Some(old) = old {
            serde_json::from_slice(old).unwrap_or_default()
        } else {
            History {
                word: word.to_string(),
                first_searched: now,
                ..Default::default()
            }
        };
        if review {
            history.last_reviewed = Some(now);
        } else {
            history.searched_count += 1;
        }
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

static REVIEW_PERIOD: Lazy<Vec<i64>> = Lazy::new(|| {
    vec![1, 3, 7, 14, 30, 60, 90, 180, 365]
        .into_iter()
        .map(|x| x * 60 * 60 * 24)
        .collect::<Vec<_>>()
});

pub fn list_reviews() -> anyhow::Result<Vec<History>> {
    let history_tree = DB.open_tree("history")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let l: Vec<History> = history_tree
        .iter()
        .filter_map(|kv| kv.ok())
        .filter_map(|kv| serde_json::from_slice::<History>(&kv.1).ok())
        .filter_map(|h| {
            let elapsed_first_searched = now - h.first_searched;
            let elapsed_review = h.last_reviewed.unwrap_or(h.first_searched) - h.first_searched;
            REVIEW_PERIOD
                .iter()
                .find(|&&d| elapsed_first_searched > d && elapsed_review < d)
                .map(|_| h)
        })
        .collect();

    for history in &l {
        save_history(&history.word, true)?;
    }

    DB.flush()?;

    Ok(l)
}
