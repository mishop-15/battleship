use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::Rng;

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
}

impl Player {
    pub fn new(id: String, is_bot: bool) -> Self {
        let mut player = Self { 
            id, 
            is_bot, 
            board: [[CellState::Empty; 10]; 10], 
            ships: Vec::new(),
            remaining_health: 17, 
        };
        player.place_random_ships();
        player
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
                Err("Already fired here!".to_string())
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
        let winner = if hits_made >= 5 {  
            self.status = GameStatus::Finished;
            self.winner = Some(player_id.clone());
            Some(player_id)
        } else {
            None
        };

        Ok((result, winner))
    }
}