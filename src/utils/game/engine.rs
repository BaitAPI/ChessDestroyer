use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::Duration;
use shakmaty::{Chess, EnPassantMode, Move};
use shakmaty::fen::Fen;
use shakmaty::uci::Uci;
use tokio::sync::mpsc::{channel, Receiver};


pub enum EngineError{
    StdinWriteError,
    StdoutReadError,
    NextMoveError
}

pub struct Engine {
    #[allow(dead_code)]
    child_guard: ChildGuard,
    sender: ChildStdin,
    receiver: Receiver<String>,
    depth: i16,
}

impl Engine {
    pub fn new(depth: i16) -> Option<Self> {
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

        Some(Engine {
            child_guard,
            sender: _stdin,
            receiver: rx,
            depth,
        })
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

        let mv: Uci = mv.parse().map_err(|_| EngineError::NextMoveError)?;
        let mov = mv.to_move(board).map_err(|_| EngineError::NextMoveError)?;
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