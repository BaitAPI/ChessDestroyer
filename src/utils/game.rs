// The `engine` module contains the logic for the chess engine.
mod engine;

use std::time::Duration;
// Importing necessary modules and structures from the `rand` and `shakmaty` crates.
use shakmaty::{Chess, Move, Position};
use rand::seq::SliceRandom;
use shakmaty::uci::Uci;

// Importing the `Engine` structure from the `engine` module.
use engine::Engine;

// Enum representing the difficulty levels of the game.
#[derive(Clone)]
pub enum DIFFICULTY {
    EASY,
    MEDIUM,
    HARD,
}

impl DIFFICULTY {
    // Method to parse the difficulty level into a depth for the chess engine.
    pub fn parse_depth(&self) -> u16 {
        match self {
            DIFFICULTY::EASY => 1,
            DIFFICULTY::MEDIUM => 3,
            DIFFICULTY::HARD => 10
        }
    }

    pub fn parse_elo(&self) -> u16{
        match self {
            DIFFICULTY::EASY => 400,
            DIFFICULTY::MEDIUM => 900,
            DIFFICULTY::HARD => 3500
        }
    }

    // Method to create a new `DIFFICULTY` from an integer.
    pub fn new(level: i16) -> Option<Self> {
        match level {
            1 => Some(DIFFICULTY::EASY),
            2 => Some(DIFFICULTY::MEDIUM),
            3 => Some(DIFFICULTY::HARD),
            _ => None
        }
    }

    // Method to parse the difficulty level into a player name.
    pub fn parse_player_name(&self) -> &'static str {
        match self {
            DIFFICULTY::EASY => "Martin",
            DIFFICULTY::MEDIUM => "Maggus Reischl",
            DIFFICULTY::HARD => "Maggus Carlsen"
        }
    }
}

// Enum representing the color of the player.
#[derive(Clone)]
pub enum COLOR {
    BLACK,
    WHITE,
}

impl COLOR {
    // Method to create a new `COLOR` from a character.
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

    // Method to parse the color into a character code.
    pub fn parse_code(&self)->char{
        match &self {
            COLOR::BLACK => 'b',
            COLOR::WHITE => 'w'
        }
    }
}

// Structure representing a game of chess.
pub struct Game {
    pub board: Chess,
    pub engine: Engine,
    #[allow(dead_code)]
    difficulty: DIFFICULTY,
    #[allow(dead_code)]
    username: String,
}

impl Game {
    // Asynchronous method to create a new `Game`.
    pub async fn new(user_color: COLOR, difficulty: DIFFICULTY, username: String) -> Option<Self> {
        let mut board = Chess::default();
        let mut engine = Engine::new(difficulty.parse_depth(), difficulty.parse_elo())?;
        if matches!(user_color, COLOR::BLACK) {
            tokio::time::sleep(Duration::from_millis(300)).await;
            let board_clone = board.clone();
            let mov = engine.gen_next_move(&board_clone).await.ok()?;
            board.play_unchecked(&mov);
        };
        Some(Game {
            board,
            engine,
            difficulty,
            username,
        })
    }
}

pub fn find_with_auto_promotion(uci: &Uci, board: &Chess)->Option<Move>{
    match uci.to_move(board) {
        Ok(mov) => Some(mov),
        Err(_) => {
            let promotions = board.promotion_moves();
            let compare = |x: &&Move| x.to().to_string() == uci.to_string()[2..] && x.from().is_some_and(|from|from.to_string() == uci.to_string()[..2]);
            let mov = promotions.iter().find(compare)?;
            Some(mov.clone())
        }
    }
}