use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::Duration;
use shakmaty::{Chess, EnPassantMode, Move};
use shakmaty::fen::Fen;
use shakmaty::uci::Uci;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::task::JoinHandle;
use crate::utils::errors::internal::InternalResult;
use crate::utils::game::find_with_auto_promotion;

pub struct Engine {
    // Dead code needs to be allowed here, because the child guard is needed to ensure subprocess kill after drop
    handle: JoinHandle<()>,
    #[allow(dead_code)]
    child_guard: ChildGuard,
    sender: ChildStdin,
    receiver: Receiver<String>,
    depth: u32,
}

impl Engine {
    pub fn new(depth: u32, elo: u16) -> Option<Self> {
        let child = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn().ok()?;
        let mut child_guard = ChildGuard { child };
        let _stdin = child_guard.child.stdin.take()?;
        let _stdout = child_guard.child.stdout.take()?;

        let (tx, rx) = channel(1024);
        let reader = BufReader::new(_stdout);

        let handle = tokio::spawn(async move {
            for line in reader.lines() {
                let line = line.unwrap_or("".to_string());
                if line.contains("bestmove") {
                    let mv = &line[9..13];
                    let _ = tx.send(String::from(mv)).await;
                }
            }
        });
        let mut engine = Engine {
            handle,
            child_guard,
            sender: _stdin,
            receiver: rx,
            depth,
        };
        engine.set_elo(elo).ok()?;
        Some(engine)
    }
    fn set_elo(&mut self, elo: u16) -> InternalResult<()> {
        let uci_limit_cmd = "setoption name UCI_LimitStrength value true".to_string();
        let uci_elo = format!("setoption name UCI_Elo value {}", elo);
        let _ = self.send(uci_limit_cmd).map_err(|_| "ENGINE: Could not send message to subprocess")?;
        let _ = self.send(uci_elo).map_err(|_| "ENGINE: Could not send message to subprocess")?;
        Ok(())
    }

    fn send(&mut self, message: String) -> InternalResult<()> {
        self.sender.write_all(format!("{}\n", message).as_bytes()).map_err(|_| "ENGINE: Could not write to stdout")
    }
    async fn receive(&mut self) -> InternalResult<String> {
        self.receiver.recv().await.ok_or("ENGINE: Could not receive Engine stdout")
    }
    pub async fn gen_next_move(&mut self, board: &Chess) -> InternalResult<Move> {
        let fen = Fen::from_position(board.clone(), EnPassantMode::Legal);
        let fen_cmd = format!("position fen {}", fen.to_string());
        let depth_cmd = format!("go depth {}", self.depth);

        let _ = self.send(fen_cmd).map_err(|_| "ENGINE: Could not send fen command")?;
        let _ = self.send(depth_cmd).map_err(|_| "ENGINE: Could not send depth command")?;
        tokio::time::sleep(Duration::from_millis(250)).await;
        let mv = self.receive().await.map_err(|_| "ENGINE: Could not receive generated move")?;

        let uci: Uci = mv.parse().map_err(|_| "ENGINE: Generated move is no valid UCI")?;
        let mov = find_with_auto_promotion(&uci, &board).ok_or("ENGINE: Generated move is not valid")?;
        Ok(mov)
    }
}

impl Drop for Engine{
    fn drop(&mut self) {
        self.handle.abort();
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