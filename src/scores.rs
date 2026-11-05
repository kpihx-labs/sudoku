use crate::engine::Difficulty;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::PathBuf, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub difficulty: String,
    pub milliseconds: u128,
    pub mistakes: u32,
    pub hints: u32,
    pub date: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Scores {
    pub entries: Vec<Score>,
}

impl Scores {
    pub fn load() -> Self {
        path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn record(
        &mut self,
        difficulty: Difficulty,
        elapsed: Duration,
        mistakes: u32,
        hints: u32,
    ) -> io::Result<()> {
        self.entries.push(Score {
            difficulty: difficulty.name().into(),
            milliseconds: elapsed.as_millis(),
            mistakes,
            hints,
            date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
        self.entries
            .sort_by_key(|entry| (entry.hints, entry.mistakes, entry.milliseconds));
        let mut counts = HashMap::new();
        self.entries.retain(|entry| {
            let count = counts.entry(entry.difficulty.clone()).or_insert(0);
            *count += 1;
            *count <= 10
        });
        self.save()
    }

    pub fn for_difficulty(&self, difficulty: Difficulty) -> Vec<&Score> {
        self.entries
            .iter()
            .filter(|entry| entry.difficulty == difficulty.name())
            .take(10)
            .collect()
    }

    fn save(&self) -> io::Result<()> {
        let Some(path) = path() else { return Ok(()) };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)
    }
}

fn path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|directory| directory.join("sudoku").join("scores.json"))
}

pub fn format_time(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
