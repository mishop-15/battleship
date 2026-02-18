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
        let bot_state = if is_bot { Some(BotState::new(difficulty)) } else { None };
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

    /// BOT LOGIC: Decides where to fire based on difficulty and previous hits
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
                    if state.difficulty == Difficulty::Hard && (r + c) % 2 != 0 {
                        continue; 
                    }
                    state.shots_fired.insert((r, c));
                    return (r, c);
                }
            }
        }
        (0, 0)
    }

    pub fn process_bot_move_result(&mut self, coords: (usize, usize), result: CellState) {
        if let Some(state) = &mut self.bot_state {
            if state.difficulty != Difficulty::Easy && result == CellState::Hit {
                let (r, c) = coords;
                state.last_hit = Some(coords);
                let adj = [(r as isize - 1, c as isize), (r as isize + 1, c as isize), 
                           (r as isize, c as isize - 1), (r as isize, c as isize + 1)];

                for (ar, ac) in adj {
                    if ar >= 0 && ar < 10 && ac >= 0 && ac < 10 {
                        let target = (ar as usize, ac as usize);
                        if !state.shots_fired.contains(&target) {
                            state.target_queue.push_back(target);
                        }
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
                if self.place_ship(Ship { id: format!("ship_{}", i), len, hits: 0, coordinates: (row, col), dir }).is_ok() {
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
            if r >= 10 || c >= 10 || self.board[r][c] != CellState::Empty {
                return Err("Placement error".to_string());
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
        match self.board[r][c] {
            CellState::Empty => { self.board[r][c] = CellState::Miss; Ok(CellState::Miss) }
            CellState::Ship => { 
                self.board[r][c] = CellState::Hit; 
                self.remaining_health -= 1; 
                Ok(CellState::Hit) 
            }
            _ => Err("Already fired here".to_string())
        }   
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameStatus { Waiting, Playing, Finished }

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
        let tid = player_1.id.clone();
        Self { 
            id: Uuid::new_v4().to_string(), 
            status: GameStatus::Waiting, 
            player_1, 
            player_2: None, 
            current_turn: tid, 
            winner: None 
        }
    }
    pub fn join_game(&mut self, player_2: Player) -> Result<(), String> {
        self.player_2 = Some(player_2);
        self.status = GameStatus::Playing;
        Ok(())
    }
    pub fn make_move(&mut self, player_id: String, target: (usize, usize)) -> Result<(CellState, Option<String>), String> {
        let opponent = if player_id == self.player_1.id {
             self.player_2.as_mut().ok_or("No opponent")?
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