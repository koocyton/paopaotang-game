mod game;
mod room;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use futures_util::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use tokio_tungstenite::tungstenite::Message;

use shared::constants::*;
use shared::protocol::*;

use room::Room;

type Rooms = Arc<Mutex<HashMap<String, Arc<Mutex<Room>>>>>;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Server running on {}", addr);
    println!("Open http://localhost:3000 in your browser");

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));

    // Serve static files and WebSocket on same port
    loop {
        let (stream, addr) = listener.accept().await.expect("Failed to accept");
        let rooms = rooms.clone();
        tokio::spawn(handle_connection(stream, addr, rooms));
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, rooms: Rooms) {
    let mut buf = [0u8; 4096];
    let n = stream.peek(&mut buf).await.unwrap_or(0);
    let request = String::from_utf8_lossy(&buf[..n]);

    if request.contains("Upgrade: websocket") || request.contains("upgrade: websocket") {
        handle_websocket(stream, addr, rooms).await;
    } else {
        handle_http(stream, &request).await;
    }
}

async fn handle_http(mut stream: TcpStream, request: &str) {
    use tokio::io::AsyncWriteExt;

    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    let www_dir = PathBuf::from("client/www");

    let (status, content_type, body) = match path {
        "/" | "/index.html" => {
            let content = tokio::fs::read_to_string(www_dir.join("index.html"))
                .await
                .unwrap_or_else(|_| "index.html not found".into());
            ("200 OK", "text/html; charset=utf-8", content)
        }
        "/style.css" => {
            let content = tokio::fs::read_to_string(www_dir.join("style.css"))
                .await
                .unwrap_or_else(|_| "".into());
            ("200 OK", "text/css; charset=utf-8", content)
        }
        "/pkg/client_bg.wasm" => {
            match tokio::fs::read(www_dir.join("pkg/client_bg.wasm")).await {
                Ok(bytes) => {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/wasm\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        bytes.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.write_all(&bytes).await;
                    return;
                }
                Err(_) => ("404 Not Found", "text/plain", "WASM not found".into()),
            }
        }
        "/pkg/client.js" => {
            let content = tokio::fs::read_to_string(www_dir.join("pkg/client.js"))
                .await
                .unwrap_or_else(|_| "JS not found".into());
            ("200 OK", "application/javascript; charset=utf-8", content)
        }
        _ => ("404 Not Found", "text/plain", "Not Found".into()),
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;
}

async fn handle_websocket(stream: TcpStream, addr: SocketAddr, rooms: Rooms) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };

    println!("New WebSocket connection: {}", addr);

    let (sink, mut stream_rx) = ws_stream.split();
    let sink = sink;

    // Wait for Join message
    let (name, sink) = loop {
        match stream_rx.next().await {
            Some(Ok(Message::Text(text))) => {
                if let Ok(ClientMsg::Join { name }) = serde_json::from_str(&text) {
                    break (name, sink);
                }
            }
            Some(Ok(_)) => continue,
            _ => return,
        }
    };

    // Find or create a room
    let room_arc = {
        let mut rooms_lock = rooms.lock().await;
        let room = rooms_lock
            .values()
            .find(|r| {
                let r = r.try_lock();
                r.map_or(false, |r| !r.game.running && r.player_count() < MAX_PLAYERS)
            })
            .cloned();

        match room {
            Some(r) => r,
            None => {
                let room_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
                let new_room = Arc::new(Mutex::new(Room::new(room_id.clone())));
                rooms_lock.insert(room_id, new_room.clone());
                new_room
            }
        }
    };

    let player_id;
    let room_id;

    // Add player to room
    {
        let mut room = room_arc.lock().await;
        player_id = room.add_player(name, sink);
        room_id = room.id.clone();

        room.send_to(
            player_id,
            &ServerMsg::Welcome {
                player_id,
                room_id: room_id.clone(),
            },
        )
        .await;

        let count = room.player_count();
        room.broadcast(&ServerMsg::Waiting {
            player_count: count,
            need: room.min_players,
        })
        .await;

        if room.is_ready() && !room.game.running {
            room.game.start();
            let map: Vec<Vec<_>> = room.game.map.iter().map(|row| row.to_vec()).collect();
            let players = room.game.player_states();
            room.broadcast(&ServerMsg::GameStart { map, players }).await;

            let room_for_tick = room_arc.clone();
            tokio::spawn(game_loop(room_for_tick));
        }
    }

    println!("Player {} joined room {}", player_id, room_id);

    // Read messages from this player
    while let Some(msg_result) = stream_rx.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMsg>(&text) {
                    let mut room = room_arc.lock().await;
                    match client_msg {
                        ClientMsg::Move { dx, dy } => {
                            room.game.set_player_movement(player_id, dx, dy);
                        }
                        ClientMsg::PlaceBomb => {
                            room.game.place_bomb(player_id);
                        }
                        ClientMsg::Join { .. } => {}
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    println!("Player {} disconnected from room {}", player_id, room_id);

    {
        let mut room = room_arc.lock().await;
        room.remove_player(player_id);
    }
}

async fn game_loop(room: Arc<Mutex<Room>>) {
    let mut interval = time::interval(Duration::from_millis(TICK_RATE_MS));

    loop {
        interval.tick().await;

        let mut r = room.lock().await;
        if r.game.finished {
            r.broadcast(&ServerMsg::GameOver {
                winner: r.game.winner,
            })
            .await;
            break;
        }

        r.game.tick();
        let state = r.game.get_state_msg();
        r.broadcast(&state).await;
    }
}
