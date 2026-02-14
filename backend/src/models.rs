use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::Rng;
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Copy)]
pub enum Direction {
    Horizontal,
    Vertical
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CellState {
    Empty,
    Hit,
    Miss,
    Ship,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotState {
    pub difficulty: Difficulty,
    pub shots_fired: HashSet<(usize, usize)>,
    pub last_hit: Option<(usize, usize)>,
    pub target_queue: VecDeque<(usize, usize)>,
}
impl BotState {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            difficulty,
            shots_fired: HashSet::new(),
            last_hit: None,
            target_queue: VecDeque::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub id: String,
    pub len: u8,
    pub hits: u8,
    pub coordinates: (usize, usize),
    pub dir: Direction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub is_bot: bool,
    pub board: [[CellState; 10]; 10],
    pub ships: Vec<Ship>,
    pub remaining_health: u8,
    pub bot_state: Option<BotState>,
}

impl Player {
    pub fn new(id: String, is_bot: bool, difficulty: Difficulty) -> Self {
        let bot_state = if is_bot {
            Some(BotState::new(difficulty)) 
        } else {
            None
        };
        let mut player = Self { 
            id, 
            is_bot, 
            board: [[CellState::Empty; 10]; 10], 
            ships: Vec::new(),
            remaining_health: 17,
            bot_state,
        };
        player.place_random_ships();
        player
    }
    pub fn get_bot_move(&mut self) -> (usize, usize) {
        if let Some(state) = &mut self.bot_state {
            while let Some(target) = state.target_queue.pop_front() {
                if !state.shots_fired.contains(&target) {
                    state.shots_fired.insert(target);
                    return target;
                }
            }
            let mut rng = rand::thread_rng();
            loop {
                let r = rng.gen_range(0..10);
                let c = rng.gen_range(0..10);

                if !state.shots_fired.contains(&(r, c)) {
                    match state.difficulty {
                        Difficulty::Easy => {
                            state.shots_fired.insert((r, c));
                            return (r, c);
                        },
                        Difficulty::Medium => {
                            state.shots_fired.insert((r, c));
                            return (r, c);
                        },
                        Difficulty::Hard => {
                            if (r + c) % 2 == 0 {
                                state.shots_fired.insert((r, c));
                                return (r, c);
                            }
                        }
                    }
                }
            }
        } else {
            (0, 0)
        }
    }

    pub fn process_bot_move_result(&mut self, coords: (usize, usize), result: CellState) {
        if let Some(state) = &mut self.bot_state {
            if state.difficulty != Difficulty::Easy && result == CellState::Hit {
                let (r, c) = coords;
                let mut is_horizontal = false;
                let mut is_vertical = false;
                if let Some((lr, lc)) = state.last_hit {
                    let dr = (r as isize - lr as isize).abs();
                    let dc = (c as isize - lc as isize).abs();
                    if dr + dc == 1 {
                        if r == lr { is_horizontal = true; } 
                        if c == lc { is_vertical = true; }   
                    }
                }
                if is_horizontal {
                    state.target_queue.retain(|&(qr, _)| qr == r);
                } else if is_vertical {
                    state.target_queue.retain(|&(_, qc)| qc == c);
                }
                state.last_hit = Some(coords);
                let mut moves = Vec::new();
                if r > 0 { moves.push((r - 1, c)); } 
                if r < 9 { moves.push((r + 1, c)); } 
                if c > 0 { moves.push((r, c - 1)); } 
                if c < 9 { moves.push((r, c + 1)); } 

                for m in moves {
                    if !state.shots_fired.contains(&m) {
                        if is_horizontal && m.0 != r { continue; }
                        if is_vertical && m.1 != c { continue; }
                        state.target_queue.push_front(m);
                    }
                }
            }
        }
    }

    pub fn place_random_ships(&mut self) {
        let ship_sizes = [5, 4, 3, 3, 2];
        let mut rng = rand::thread_rng();

        for (i, &len) in ship_sizes.iter().enumerate() {
            loop {
                let dir = if rng.gen_bool(0.5) { Direction::Horizontal } else { Direction::Vertical };
                let row = rng.gen_range(0..10);
                let col = rng.gen_range(0..10);

                let temp_ship = Ship {
                    id: format!("ship_{}", i),
                    len,
                    hits: 0,
                    coordinates: (row, col),
                    dir,
                };
                if self.place_ship(temp_ship).is_ok() {
                    break;
                }
            }
        }
    }
    pub fn place_ship(&mut self, ship: Ship) -> Result<(), String> {
        let (start_row, start_col) = ship.coordinates;
        let len = ship.len as usize;
        for i in 0..len {
            let (r, c) = match ship.dir {
                Direction::Horizontal => (start_row, start_col + i),
                Direction::Vertical => (start_row + i, start_col), 
            };
            
            if r >= 10 || c >= 10 {
                return Err("Ship goes out of bounds.".to_string());
            }
            if self.board[r][c] != CellState::Empty {
                return Err(format!("Collision at {},{}", r, c));
            }
        }
        for i in 0..len {
            let (r, c) = match ship.dir {
                Direction::Horizontal => (start_row, start_col + i),
                Direction::Vertical => (start_row + i, start_col),
            };
            self.board[r][c] = CellState::Ship;
        }
        self.ships.push(ship);
        Ok(())
    }

    pub fn receive_shot(&mut self, coord: (usize, usize)) -> Result<CellState, String> {
        let (r, c) = coord;
        if r >= 10 || c >= 10 {
            return Err("shot out of bounds".to_string());
        }
        match self.board[r][c] {
            CellState::Empty => {
                self.board[r][c] = CellState::Miss;
                Ok(CellState::Miss)
            }
            CellState::Ship => {
                self.board[r][c] = CellState::Hit;
                self.remaining_health -= 1; 
                Ok(CellState::Hit)
            }
            CellState::Hit | CellState::Miss => {
                Err("already fired here!".to_string())
            }
        }   
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    Waiting,   
    Playing,    
    Finished,    
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub status: GameStatus,
    pub player_1: Player,
    pub player_2: Option<Player>, 
    pub current_turn: String,      
    pub winner: Option<String>,   
}

impl Game {
    pub fn new(player_1: Player) -> Self {
        let first_turn = player_1.id.clone();
        Self { 
            id: Uuid::new_v4().to_string(), 
            status: GameStatus::Waiting, 
            player_1, 
            player_2: None, 
            current_turn: first_turn, 
            winner: None 
        }
    }

    pub fn join_game(&mut self, player_2: Player) -> Result<(), String> {
        if self.player_2.is_none() {
            self.player_2 = Some(player_2);
            self.status = GameStatus::Playing;
            Ok(())
        } else {
            Err("game full".to_string())
        }
    }
    pub fn make_move(&mut self, player_id: String, target: (usize, usize)) -> Result<(CellState, Option<String>), String> {
        let opponent = if player_id == self.player_1.id {
             self.player_2.as_mut().ok_or("Player 2 missing")?
        } else {
             &mut self.player_1
        };
        let result = opponent.receive_shot(target)?;
        let hits_made = 17 - opponent.remaining_health;
        if hits_made >= 7 {
            self.status = GameStatus::Finished;
            self.winner = Some(player_id.clone());
            return Ok((result, Some(player_id)));
        }
        Ok((result, None))
    }
}