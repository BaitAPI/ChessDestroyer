mod engine;

use rand::seq::SliceRandom;
use shakmaty::{Chess, Position};
use engine::Engine;

#[derive(Clone)]
pub enum DIFFICULTY {
    EASY,
    MEDIUM,
    HARD,
}

impl DIFFICULTY {
    pub fn parse_depth(&self) -> i16 {
        match self {
            DIFFICULTY::EASY => 1,
            DIFFICULTY::MEDIUM => 3,
            DIFFICULTY::HARD => 10
        }
    }
    pub fn new(level: i16) -> Option<Self> {
        match level {
            1 => Some(DIFFICULTY::EASY),
            2 => Some(DIFFICULTY::MEDIUM),
            3 => Some(DIFFICULTY::HARD),
            _ => None
        }
    }
    pub fn parse_player_name(&self) -> &'static str {
        match self {
            DIFFICULTY::EASY => "Jeff",
            DIFFICULTY::MEDIUM => "Bezos",
            DIFFICULTY::HARD => "Fun"
        }
    }
}

#[derive(Clone)]
pub enum COLOR {
    BLACK,
    WHITE,
}

impl COLOR {
    pub fn new(character: char) -> Option<Self> {
        match character {
            'b' => Some(COLOR::BLACK),
            'w' => Some(COLOR::WHITE),
            'r' => {
                let mut rng = rand::thread_rng();
                let choices = [COLOR::BLACK, COLOR::WHITE];
                choices.choose(&mut rng).cloned()
            }
            _ => None
        }
    }
    pub fn parse_code(&self)->char{
        match &self {
            COLOR::BLACK => 'b',
            COLOR::WHITE => 'w'
        }
    }
}


pub struct Game {
    pub board: Chess,
    pub engine: Engine,
    #[allow(dead_code)]
    difficulty: DIFFICULTY,
    #[allow(dead_code)]
    username: String,
}

impl Game {
    pub async fn new(user_color: COLOR, difficulty: DIFFICULTY, username: String) -> Option<Self> {
        let mut board = Chess::default();
        let mut engine = Engine::new(difficulty.parse_depth())?;
        if matches!(user_color, COLOR::BLACK) {
            let mv = engine.gen_next_move(&board.clone()).await.ok()?;
            board = board.play(&mv).ok()?;
        };
        Some(Game {
            board,
            engine,
            difficulty,
            username,
        })
    }
}

