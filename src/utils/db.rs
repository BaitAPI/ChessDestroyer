use std::env::current_dir;
use std::sync::{Arc, Mutex, MutexGuard};
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use crate::utils::game::DIFFICULTY;

#[derive(Serialize)]
pub struct ScoreEntry {
    pub winner: String,
    pub score: f32
}

impl ScoreEntry{
    pub fn new(winner: &String, moves: u32, difficulty: &DIFFICULTY) ->Self{
        let score = Self::calc_score(moves, difficulty);
        ScoreEntry{
            winner: winner.clone(),
            score
        }
    }
    fn calc_score(moves: u32, difficulty: &DIFFICULTY) ->f32{
        (moves * difficulty.parse_depth()) as f32
    }
}

pub struct DB {
    conn: Arc<Mutex<Connection>>,
}
impl DB{
    pub fn new(path: &str)->Result<Self, ()>{
        let binding = current_dir().map_err(|_|())?;
        let dir_path = binding.to_str().ok_or(())?;
        let conn = Connection::open_with_flags(format!("{}{}", dir_path, path), OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE).map_err(|_|())?;
        Ok(DB{
            conn: Arc::new(Mutex::new(conn))
        })
    }
    pub fn get(&self)->Result<MutexGuard<Connection>,()>{
        let conn = self.conn.lock().map_err(|_|())?;
        Ok(conn)
    }
}

pub fn set_score_schema(conn: &Connection)->Result<(), ()>{
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS Score (
            winner TEXT PRIMARY KEY,
            score FLOAT
        )",
        (), // empty list of parameters.
    ).map_err(|_|())?;
    Ok(())
}
pub fn add_score_entry(conn: &Connection, entry: ScoreEntry)->Result<(),()>{
    let res = conn.execute(
        "INSERT INTO Score (winner, score) VALUES (?1, ?2)",
        (&entry.winner, &entry.score)
    );
    match res {
        Ok(_) => Ok(()),
        Err(_) => {
            let old_entry = find_entry(conn, &entry.winner).ok_or(())?;
            if old_entry.score > entry.score{
                conn.execute(
                    "UPDATE Score SET score = ?1 WHERE winner = ?2",
                    (entry.score, entry.winner),
                ).map_err(|_|())?;
            }
            Ok(())
        }
    }
}

fn find_entry(conn: &Connection, winner: &str)->Option<ScoreEntry>{
    let query = "SELECT winner, score FROM Score WHERE winner IS ?1 LIMIT 1";
    let mut stmt = conn.prepare(&query).ok()?;

    let mut score_iter = stmt.query_map([winner], |row|{
        Ok(
            ScoreEntry{
                winner: row.get(0)?,
                score: row.get(1)?
            }
        )
    }).ok()?;
    score_iter.next()?.ok()
}

pub fn get_top(conn: &Connection, count: u16)->Result<Vec<ScoreEntry>, ()>{
    let query = format!("SELECT winner, score FROM Score ORDER BY score ASC LIMIT {}", count);
    let mut stmt = conn.prepare(&query).map_err(|_|())?;

    let mut entries = Vec::new();
    let iterator = stmt.query_map([], |row|{
        Ok(
            ScoreEntry{
                winner: row.get(0)?,
                score: row.get(1)?
            }
        )
    }).map_err(|_|())?;

    for score in iterator{
        let score = score.map_err(|_|())?;
        entries.push(score);
    }

    Ok(entries)
}