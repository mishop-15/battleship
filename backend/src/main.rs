use axum::{
    extract::{Path, State, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use std::net::SocketAddr;
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use rand::Rng; 

mod models;
use models::{Game, Player};

type GameStore = Arc<Mutex<HashMap<String, Game>>>;

#[derive(Clone)]
struct AppState {
    games: GameStore,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        games: Arc::new(Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        .route("/", get(health_check))
        .route("/create_game", post(create_game_handler))
        .route("/ws/:game_id", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str { "Battleship Server Running" }

async fn create_game_handler(State(state): State<AppState>) -> Json<Value> {
    let player_1 = Player::new("User".to_string(), false);
    let mut new_game = Game::new(player_1);
    let bot = Player::new("Bot".to_string(), true);
    let _ = new_game.join_game(bot);

    let game_id = new_game.id.clone();
    
    {
        let mut games = state.games.lock().unwrap();
        games.insert(game_id.clone(), new_game);
    }

    Json(json!({ "status": "created", "game_id": game_id }))
}

async fn ws_handler(ws: WebSocketUpgrade, Path(game_id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let exists = state.games.lock().unwrap().contains_key(&game_id);
    if !exists { return "Game not found".into_response(); }
    
    ws.on_upgrade(move |socket| handle_game_socket(socket, game_id, state))
}

async fn handle_game_socket(mut socket: WebSocket, game_id: String, state: AppState) {
    let my_id = "User".to_string(); 
    let init_msg = {
        let games = state.games.lock().unwrap();
        if let Some(game) = games.get(&game_id) {
            Some(json!({ "type": "init", "board": game.player_1.board }).to_string())
        } else { None }
    };
    if let Some(msg) = init_msg { let _ = socket.send(Message::Text(msg)).await; }
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let parts: Vec<&str> = text.split(',').collect();
            if parts.len() != 2 { continue; }
            let r: usize = parts[0].parse().unwrap_or(0);
            let c: usize = parts[1].parse().unwrap_or(0);
            let response = {
                let mut games = state.games.lock().unwrap();
                if let Some(game) = games.get_mut(&game_id) {
                    
                    match game.make_move(my_id.clone(), (r, c)) {
                        Ok((user_res, winner)) => {
                            let mut bot_data = None;
                            if winner.is_none() {
                                let mut rng = rand::thread_rng();
                                let mut bot_r = rng.gen_range(0..10);
                                let mut bot_c = rng.gen_range(0..10);
                                
                                for _ in 0..10 {
                                    if let Ok((b_res, b_win)) = game.make_move("Bot".to_string(), (bot_r, bot_c)) {
                                        bot_data = Some((bot_r, bot_c, b_res, b_win));
                                        break;
                                    }
                                    bot_r = rng.gen_range(0..10);
                                    bot_c = rng.gen_range(0..10);
                                }
                            }

                            Some(json!({
                                "status": "success",
                                "turn_update": {
                                    "user": { "row": r, "col": c, "result": user_res },
                                    "bot": bot_data.as_ref().map(|(br, bc, bres, _)| {
                                        json!({ "row": br, "col": bc, "result": bres })
                                    }),
                                    "winner": winner.or(bot_data.and_then(|(_,_,_,w)| w))
                                }
                            }))
                        },
                        Err(e) => {
                            Some(json!({
                                "status": "error",
                                "message": e
                            }))
                        }
                    }
                } else {
                    None 
                }
            };

            if let Some(resp) = response {
                let _ = socket.send(Message::Text(resp.to_string())).await;
            }
        }
    }
}