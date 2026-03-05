use rand::seq::IndexedRandom as _;

const THREE_ROWS: [(usize, usize, usize); 8] = [
    (0,1,2), (3,4,5), (6,7,8),
    (0,3,6), (1,4,7), (2,5,8),
    (0,4,8), (2,4,6),
];

// algorithm i/o; can have any number of inputs (all must be in the board), and any
// number of possible outputs that it will randomly select from
type InputOutput = &'static [(&'static [u8], &'static [u8])];

// i/o strategies
const CHECKMATES: InputOutput = &[
    // diagonals
    (&[0,4],&[8]), (&[0,8],&[4]), (&[4,8],&[0]),
    (&[2,4],&[6]), (&[2,6],&[4]), (&[4,6],&[2]),
    
    // rows
    (&[0,1],&[2]), (&[0,2],&[1]), (&[1,2],&[0]),
    (&[3,4],&[5]), (&[3,5],&[4]), (&[4,5],&[3]),
    (&[6,7],&[8]), (&[6,8],&[7]), (&[7,8],&[6]),
    
    // columns
    (&[0,3],&[6]), (&[0,6],&[3]), (&[3,6],&[0]),
    (&[1,4],&[7]), (&[1,7],&[4]), (&[4,7],&[1]),
    (&[2,5],&[8]), (&[2,8],&[5]), (&[5,8],&[2]),
];

const FIRST_MOVES: InputOutput = &[
    // center, pick random corner
    (&[4],&[0,2,6,8]),
    
    // corner, pick center
    (&[0],&[4]),
    (&[2],&[4]),
    (&[6],&[4]),
    (&[8],&[4]),
    
    // side, pick center
    (&[1],&[4]),
    (&[3],&[4]),
    (&[5],&[4]),
    (&[7],&[4]),
];

// counter-strats for more sophisticated strats that slightly increase
// a human player's chances of winning
const COUNTER_STRATS: InputOutput = &[
    // corner to opposite corner, pick a random side
    (&[0,8],&[1,3,5,7]),
    (&[2,6],&[1,3,5,7]),
    
    // side to corner
    (&[5,0],&[2]),(&[5,6],&[8]),
    (&[7,0],&[2]),(&[7,2],&[8]),
    (&[3,2],&[0]),(&[3,8],&[6]),
    (&[1,6],&[0]),(&[1,8],&[2]),
    
    // side to next side
    (&[1,5],&[2]),
    (&[5,7],&[8]),
    (&[7,3],&[6]),
    (&[3,1],&[0]),
    
    // center to corner
    (&[4,0],&[2,6]),
    (&[4,2],&[8,0]),
    (&[4,6],&[8,0]),
    (&[4,8],&[2,6]),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    X,
    O,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Difficulty {
    Easy = 0,
    Difficult = 1,
    GoodLuck = 2,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "Easy"),
            Difficulty::Difficult => write!(f, "Difficult"),
            Difficulty::GoodLuck => write!(f, "Good Luck"),
        }
    }
}

impl Difficulty {
    pub fn all() -> &'static [Difficulty] {
        &[Difficulty::Easy, Difficulty::Difficult, Difficulty::GoodLuck]
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Empty => write!(f, " "),
            Cell::X => write!(f, "X"),
            Cell::O => write!(f, "O"),
        }
    }
}

pub struct Board {
    pub difficulty: Difficulty,
    pub cells: [Cell; 9],
}

impl Board {
    pub fn new(difficulty: Difficulty) -> Self {
        Board {
            difficulty,
            cells: [Cell::Empty; 9],
        }
    }
    
    pub fn calculate_winner(&self) -> Option<Cell> {
        for pattern in THREE_ROWS {
            if self.cells[pattern.0] == self.cells[pattern.1]
                && self.cells[pattern.1] == self.cells[pattern.2]
                && self.cells[pattern.0] != Cell::Empty
            {
                return Some(self.cells[pattern.0]);
            }
        }
        None
    }
    
    pub fn game_over(&self) -> bool {
        self.calculate_winner().is_some() || self.cells.iter().all(|&cell| cell != Cell::Empty)
    }
    
    fn find_match(&self, io: InputOutput, target: Cell) -> Vec<u8> {
        for (inputs, outputs) in io {
            if inputs.iter().all(|&i| self.cells[i as usize] == target) {
                let available: Vec<u8> = outputs.iter()
                    .filter(|&&i| self.cells[i as usize] == Cell::Empty)
                    .copied()
                    .collect();
                
                if !available.is_empty() {
                    return available;
                }
            }
        }
        vec![]
    }
    
    pub fn calculate_move(&self) -> Option<u8> {
        let mut rng = rand::rng();
        let x_moves = self.cells.iter().filter(|&&cell| cell == Cell::X).count();
        let o_moves = self.cells.iter().filter(|&&cell| cell == Cell::O).count();
        
        if self.game_over() {
            return None;
        }
        
        if self.difficulty == Difficulty::Easy {
            // perform purely random moves
            self.cells.iter()
                .enumerate()
                .filter(|(_, c)| **c == Cell::Empty)
                .map(|(i, _)| i as u8)
                .collect::<Vec<u8>>()
                .choose(&mut rng)
                .copied()
        } else {
            let mut choices = if x_moves == 1 && o_moves == 0 {
                self.find_match(FIRST_MOVES, Cell::X)
            } else {
                let win = self.find_match(CHECKMATES, Cell::O);
                if !win.is_empty() {
                    win
                } else {
                    let block = self.find_match(CHECKMATES, Cell::X);
                    if !block.is_empty() {
                        block
                    } else {
                        // don't perform counter strats if difficulty < GoodLuck
                        if self.difficulty >= Difficulty::GoodLuck {
                            self.find_match(COUNTER_STRATS, Cell::X)
                        } else {
                            vec![]
                        }
                    }
                }
            };
            
            if choices.is_empty() {
                choices = self.cells.iter()
                    .enumerate()
                    .filter(|(_, c)| **c == Cell::Empty)
                    .map(|(i, _)| i as u8)
                    .collect();
            }
    
            choices.choose(&mut rng).copied()
        }
    }
    
    pub fn play(&mut self, index: u8) {
        if self.game_over() || self.cells[index as usize] != Cell::Empty {
            return;
        }
        self.cells[index as usize] = Cell::X;
        if let Some(index) = self.calculate_move() {
            self.cells[index as usize] = Cell::O;
        }
    }
    
    pub fn reset(&mut self) {
        self.cells = [Cell::Empty; 9];
    }
}
