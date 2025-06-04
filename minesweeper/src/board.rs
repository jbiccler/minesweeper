use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Write};
use std::vec;
use std::{collections::HashMap, collections::HashSet};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

type Position = (usize, usize);
const DIRS: [(isize, isize); 8] = [
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, 1),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Init,
    OnGoing,
    Lost,
    Won,
}

#[derive(Debug)]
pub enum OpenError {
    AlreadyOpen,
    AlreadyFlagged,
    AlreadyLost,
    AlreadyWon,
    MinesNotInit,
    OutOfBounds,
}
#[derive(Debug)]
pub enum FlagError {
    AlreadyOpen,
    AlreadyLost,
    AlreadyWon,
    MinesNotInit,
    OutOfBounds,
}

#[derive(Debug, Clone, Copy)]
pub enum Square {
    Mine,
    Opened(u8),
    Flag,
    NotYetOpened,
}

pub struct Board {
    pub rows: usize,
    pub cols: usize,
    pub nr_mines: usize,
    mines: Option<HashSet<Position>>,
    pub open_fields: HashSet<Position>,
    pub flagged_fields: HashSet<Position>,
    pub counts: HashMap<Position, u8>,
    pub state: GameState,
}

impl Board {
    pub fn new(rows: usize, cols: usize, nr_mines: usize) -> Board {
        assert!(rows * cols > nr_mines);

        Board {
            rows,
            cols,
            nr_mines,
            mines: None,
            flagged_fields: HashSet::new(),
            open_fields: HashSet::new(),
            counts: HashMap::new(),
            state: GameState::Init,
        }
    }

    fn reset_board(&mut self) {
        self.flagged_fields.clear();
        self.open_fields.clear();
        self.counts.clear();
        self.state = GameState::Init;
        self.mines = None;
    }

    pub fn lost(&self) -> bool {
        matches!(self.state, GameState::Lost)
    }

    pub fn ongoing(&self) -> bool {
        matches!(self.state, GameState::OnGoing)
    }

    pub fn initialized(&self) -> bool {
        !matches!(self.state, GameState::Init)
    }

    pub fn init_mines(&mut self, start_position: Position, seed: Option<u64>) {
        let mut rng = if let Some(seed) = seed {
            // Seed the random generator
            ChaCha8Rng::seed_from_u64(seed)
        } else {
            // Get fresh seed directly from OS
            ChaCha8Rng::from_os_rng()
        };

        let mut mines = HashSet::new();
        while mines.len() < self.nr_mines {
            let x: usize = rng.random_range(0..self.cols);
            let y: usize = rng.random_range(0..self.rows);
            if (x, y) != start_position {
                mines.insert((x, y));
            }
        }
        self.reset_board();
        self.mines = Some(mines);
        self.state = GameState::OnGoing;
        self.set_counts();
        self.open(start_position).unwrap();
    }

    pub fn open(&mut self, pos: Position) -> Result<GameState, OpenError> {
        match self.state {
            GameState::Lost => Err(OpenError::AlreadyLost),
            GameState::Init => Err(OpenError::MinesNotInit),
            GameState::Won => Err(OpenError::AlreadyWon),
            GameState::OnGoing => {
                if pos.0 >= self.cols || pos.1 >= self.rows {
                    Err(OpenError::OutOfBounds)
                } else if self.mines.as_ref().unwrap().contains(&pos) {
                    self.state = GameState::Lost;
                    Ok(GameState::Lost)
                } else if self.flagged_fields.contains(&pos) {
                    Err(OpenError::AlreadyFlagged)
                } else if self.open_fields.insert(pos) {
                    // did not contain pos yet -> update
                    // if this field has a zero count, then open neighboring fields also
                    if !self.counts.contains_key(&pos) {
                        let mut to_open = vec![];
                        let mut next: BTreeSet<Position> = self
                            .iter_neighbors(pos)
                            .filter(|p| !self.open_fields.contains(p))
                            .collect();
                        let mut seen = Vec::with_capacity(next.len());

                        while !next.is_empty() {
                            let n = next.pop_first().unwrap();
                            if seen.contains(&n) {
                                continue;
                            }
                            seen.push(n);
                            if self.mines.as_ref().unwrap().contains(&n) {
                                // pass, don't open a mine
                            } else if !self.open_fields.contains(&n) {
                                if self.counts.contains_key(&n) {
                                    // mine count > 0 -> stop here as new frontier
                                    to_open.push(n);
                                } else {
                                    // zero count -> iterate over neighbors again
                                    to_open.push(n);
                                    for i in self.iter_neighbors(n) {
                                        if !seen.contains(&i) && !self.open_fields.contains(&i) {
                                            next.insert(i);
                                        }
                                    }
                                }
                            }
                        }
                        for p in to_open {
                            self.open_fields.insert(p);
                        }
                    }
                    if self.check_win_condition() == GameState::Won {
                        self.state = GameState::Won;
                        Ok(GameState::Won)
                    } else {
                        Ok(GameState::OnGoing)
                    }
                } else {
                    // pos already contained -> don't update
                    Err(OpenError::AlreadyOpen)
                }
            }
        }
    }

    pub fn flag(&mut self, pos: Position) -> Result<GameState, FlagError> {
        match self.state {
            GameState::Lost => Err(FlagError::AlreadyLost),
            GameState::Init => Err(FlagError::MinesNotInit),
            GameState::Won => Err(FlagError::AlreadyWon),
            GameState::OnGoing => {
                if pos.0 >= self.cols || pos.1 >= self.rows {
                    Err(FlagError::OutOfBounds)
                } else if self.open_fields.contains(&pos) {
                    // field is already open, can't be flagged.
                    Err(FlagError::AlreadyOpen)
                } else if self.flagged_fields.contains(&pos) {
                    // unflag
                    self.flagged_fields.remove(&pos);
                    Ok(GameState::OnGoing)
                } else {
                    self.flagged_fields.insert(pos);
                    if self.check_win_condition() == GameState::Won {
                        self.state = GameState::Won;
                        Ok(GameState::Won)
                    } else {
                        Ok(GameState::OnGoing)
                    }
                }
            }
        }
    }

    fn check_win_condition(&self) -> GameState {
        match self.state {
            GameState::OnGoing => {
                if self.flagged_fields.len() == self.nr_mines
                    && self.open_fields.len() + self.flagged_fields.len() == self.cols * self.rows
                {
                    if self.flagged_fields == *self.mines.as_ref().unwrap() {
                        GameState::Won
                    } else {
                        GameState::OnGoing
                    }
                } else {
                    GameState::OnGoing
                }
            }
            s => s,
        }
    }

    fn set_counts(&mut self) {
        self.counts.clear();
        // iterate over mines, find their neighbors and count
        for &m in self.mines.as_ref().unwrap().iter() {
            let neighs = self.iter_neighbors(m);
            for n in neighs {
                self.counts.entry(n).and_modify(|c| *c += 1).or_insert(1);
            }
        }
    }

    pub fn iter_neighbors(&self, (x, y): Position) -> impl Iterator<Item = Position> {
        let (r, c) = (self.rows as isize, self.cols as isize);
        let x = x as isize;
        let y = y as isize;
        DIRS.iter()
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(move |(nx, ny)| {
                *nx >= 0 && *nx < c && *ny >= 0 && *ny < r && (*nx, *ny) != (x, y)
            })
            .map(|(nx, ny)| (nx as usize, ny as usize))
    }

    fn _neighboring_mines(&self, pos: Position) -> u8 {
        self.iter_neighbors(pos)
            .filter(|pos| self.mines.as_ref().unwrap().contains(pos))
            .count() as u8
    }

    pub fn get_board_state(&self) -> Vec<Vec<Square>> {
        let mut map = vec![vec![Square::NotYetOpened; self.cols]; self.rows];
        if self.state == GameState::Init {
            return map;
        }
        for (x, y) in self.open_fields.iter() {
            map[*y][*x] = Square::Opened(self.counts.get(&(*x, *y)).unwrap_or(&0u8).to_owned());
        }
        if self.state == GameState::Lost {
            for (x, y) in self.mines.as_ref().unwrap().iter() {
                map[*y][*x] = Square::Mine;
            }
        }
        for (x, y) in self.flagged_fields.iter() {
            map[*y][*x] = Square::Flag;
        }
        map
    }

    pub fn get_frontier(&self) -> HashSet<Position> {
        let mut frontier = HashSet::new();
        for &open in self.open_fields.iter() {
            let neighbors = self.iter_neighbors(open);
            for n in neighbors {
                if !self.open_fields.contains(&n) {
                    frontier.insert(open);
                    break;
                }
            }
        }
        frontier
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.rows {
            for x in 0..self.cols {
                let pos = (x, y);

                if !self.open_fields.contains(&pos) {
                    if self.flagged_fields.contains(&pos) {
                        f.write_str("🚩 ")?;
                    } else if self.mines.as_ref().unwrap().contains(&pos) {
                        f.write_str("💣 ")?;
                    } else {
                        f.write_str("🟪 ")?;
                    }
                } else if self.mines.as_ref().unwrap().contains(&pos) {
                    f.write_str("💣 ")?;
                } else {
                    let mine_count = self.counts.get(&pos).unwrap_or(&0).to_owned();
                    write!(f, " {} ", mine_count)?;
                    // f.write_str("⬜ ")?;
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.state {
            GameState::Init => {
                for _ in 0..self.rows {
                    for _ in 0..self.cols {
                        f.write_str("🟪 ")?;
                    }
                }
            }
            GameState::OnGoing => {
                for y in 0..self.rows {
                    for x in 0..self.cols {
                        let pos = (x, y);
                        if !self.open_fields.contains(&pos) {
                            if self.flagged_fields.contains(&pos) {
                                f.write_str("🚩 ")?;
                            } else {
                                f.write_str("🟪 ")?;
                            }
                        } else {
                            let mine_count = self.counts.get(&pos).unwrap_or(&0).to_owned();
                            write!(f, " {} ", mine_count)?;
                        }
                    }
                    f.write_char('\n')?;
                }
            }
            GameState::Lost | GameState::Won => {
                for y in 0..self.rows {
                    for x in 0..self.cols {
                        let pos = (x, y);

                        if !self.open_fields.contains(&pos) {
                            if self.flagged_fields.contains(&pos) {
                                f.write_str("🚩 ")?;
                            } else if self.mines.as_ref().unwrap().contains(&pos) {
                                f.write_str("💣 ")?;
                            } else {
                                f.write_str("🟪 ")?;
                            }
                        } else if self.mines.as_ref().unwrap().contains(&pos) {
                            f.write_str("💣 ")?;
                        } else {
                            let mine_count = self.counts.get(&pos).unwrap_or(&0).to_owned();
                            write!(f, " {} ", mine_count)?;
                        }
                    }
                    f.write_char('\n')?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_board_9_9_10(start_position: Position, seed: u64) -> Board {
        let mut board = Board::new(9, 9, 10);
        board.init_mines(start_position, Some(seed));
        board
    }

    #[test]
    fn test_mines() {
        let board = setup_board_9_9_10((0, 0), 1);
        println!("{:?}", board);
        let mut v = Vec::from_iter(board.mines.as_ref().unwrap().clone());
        v.sort();
        let expected: Vec<(usize, usize)> = vec![
            (0, 7),
            (1, 5),
            (1, 6),
            (3, 1),
            (4, 3),
            (4, 4),
            (6, 1),
            (7, 2),
            (8, 0),
            (8, 6),
        ];
        println!("{:?}", v);
        assert_eq!(v, expected);
    }
    #[test]
    fn test_neighbors() {
        let board = setup_board_9_9_10((0, 0), 1);
        let neigh_board_corner = board.iter_neighbors((0, 0));
        let neigh_middle = board.iter_neighbors((4, 4));
        let neigh_edge = board.iter_neighbors((0, 4));
        assert_eq!(neigh_board_corner.count(), 3);
        assert_eq!(neigh_middle.count(), 8);
        assert_eq!(neigh_edge.count(), 5);
    }

    #[test]
    fn test_open_clear_field() {
        let mut board = setup_board_9_9_10((0, 0), 1);
        println!("{:?}", board);
        board.open((0, 5)).unwrap();
        println!("{:?}", board);
        board.open((4, 2)).unwrap();
        println!("{:?}", board);
        board.open((5, 7)).unwrap();
        println!("{:?}", board);
    }
    #[test]
    fn test_open_already_open_field() {
        let mut board = setup_board_9_9_10((0, 0), 1);
        println!("{:?}", board);
        let err = board.open((0, 1));
        match err {
            Ok(_) => panic!("Expected an error, but got OK"),
            Err(OpenError::AlreadyOpen) => {} // success
            Err(_) => panic!("Wrong error type returned"),
        }
    }

    #[test]
    fn test_open_bomb() {
        let mut board = setup_board_9_9_10((0, 0), 1);
        println!("{:?}", board);
        let err = board.open((3, 1));
        match err {
            Ok(GameState::Lost) => {}
            _ => panic!("Wrong gamestate returned"),
        }
    }
}
