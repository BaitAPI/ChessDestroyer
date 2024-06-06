use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::Duration;
use shakmaty::{Chess, EnPassantMode, Move};
use shakmaty::fen::Fen;
use shakmaty::uci::Uci;
use tokio::sync::mpsc::{channel, Receiver};
use crate::utils::game::find_with_auto_promotion;


pub enum EngineError{
    StdinWriteError,
    StdoutReadError,
    NextMoveError,
    PromotionError
}

pub struct Engine {
    #[allow(dead_code)]
    child_guard: ChildGuard,
    sender: ChildStdin,
    receiver: Receiver<String>,
    depth: u16
}

impl Engine {
    pub fn new(depth: u16, elo: u16) -> Option<Self> {
        let child = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn().ok()?;
        let mut child_guard = ChildGuard { child };
        let _stdin = child_guard.child.stdin.take()?;
        let _stdout = child_guard.child.stdout.take()?;

        let (tx, rx) = channel(1024);
        let reader = BufReader::new(_stdout);

        tokio::spawn(async move {
            for line in reader.lines() {
                let line = line.unwrap_or("".to_string());
                if line.contains("bestmove") {
                    let mv = &line[9..13];
                    let _ = tx.send(String::from(mv)).await;
                }
            }
        });
        let mut engine = Engine {
            child_guard,
            sender: _stdin,
            receiver: rx,
            depth,
        };
        engine.set_elo(elo).ok()?;
        Some(engine)
    }
    fn set_elo(&mut self, elo: u16) ->Result<(),()>{
        let uci_limit_cmd = "setoption name UCI_LimitStrength value true".to_string();
        let uci_elo = format!("setoption name UCI_Elo value {}", elo);
        let _ = self.send(uci_limit_cmd).map_err(|_|())?;
        let _ = self.send(uci_elo).map_err(|_|())?;
        Ok(())
    }

    fn send(&mut self, message: String) -> Result<(), EngineError> {
        self.sender.write_all(format!("{}\n", message).as_bytes()).map_err(|_| EngineError::StdinWriteError)
    }
    async fn receive(&mut self) -> Result<String, EngineError> {
        self.receiver.recv().await.ok_or(EngineError::StdoutReadError)
    }
    pub async fn gen_next_move(&mut self, board: &Chess) -> Result<Move, EngineError> {
        let fen = Fen::from_position(board.clone(), EnPassantMode::Legal);
        let fen_cmd = format!("position fen {}", fen.to_string());
        let depth_cmd = format!("go depth {}", self.depth);

        let _ = self.send(fen_cmd).map_err(|_| EngineError::NextMoveError)?;
        let _ = self.send(depth_cmd).map_err(|_| EngineError::NextMoveError)?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mv = self.receive().await.map_err(|_| EngineError::NextMoveError)?;

        let uci: Uci = mv.parse().map_err(|_| EngineError::NextMoveError)?;
        let mov = find_with_auto_promotion(&uci, &board).ok_or(EngineError::PromotionError)?;
        Ok(mov)
    }
}


struct ChildGuard {
    child: Child,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}