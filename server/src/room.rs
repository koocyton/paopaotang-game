use std::collections::HashMap;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use shared::protocol::*;

use crate::game::GameInstance;

type WsSink = SplitSink<WebSocketStream<TcpStream>, Message>;

pub struct Room {
    pub id: String,
    pub game: GameInstance,
    pub sinks: HashMap<u8, Mutex<WsSink>>,
    pub next_player_id: u8,
    pub min_players: usize,
}

impl Room {
    pub fn new(id: String) -> Self {
        Room {
            id,
            game: GameInstance::new(),
            sinks: HashMap::new(),
            next_player_id: 0,
            min_players: 2,
        }
    }

    pub fn add_player(&mut self, name: String, sink: WsSink) -> u8 {
        let pid = self.next_player_id;
        self.next_player_id += 1;
        self.game.add_player(pid, name);
        self.sinks.insert(pid, Mutex::new(sink));
        pid
    }

    pub fn player_count(&self) -> usize {
        self.sinks.len()
    }

    pub fn is_ready(&self) -> bool {
        self.player_count() >= self.min_players
    }

    pub async fn broadcast(&self, msg: &ServerMsg) {
        let json = serde_json::to_string(msg).unwrap();
        for sink in self.sinks.values() {
            let mut s = sink.lock().await;
            let _ = s.send(Message::Text(json.clone())).await;
        }
    }

    pub async fn send_to(&self, player_id: u8, msg: &ServerMsg) {
        let json = serde_json::to_string(msg).unwrap();
        if let Some(sink) = self.sinks.get(&player_id) {
            let mut s = sink.lock().await;
            let _ = s.send(Message::Text(json)).await;
        }
    }

    pub fn remove_player(&mut self, player_id: u8) {
        self.sinks.remove(&player_id);
        if let Some(p) = self.game.players.iter_mut().find(|p| p.id == player_id) {
            p.alive = false;
        }
    }
}
