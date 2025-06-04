use std::io;

use clap::Parser;
use minesweeper::board::*;
use minesweeper::config::Args;
use regex::Regex;

fn main() {
    let args = Args::parse();
    if let Some(seed) = args.get_seed() {
        println!("Seed: {seed}");
    }
    println!(
        "Rows: {}, Cols: {}, Mines: {}",
        args.get_rows(),
        args.get_cols(),
        args.get_mines()
    );
    let re_open = Regex::new(r"\(?(?<x>\d+)(,|\s+)(?<y>\d+)\)?").unwrap();
    let re_flag = Regex::new(r"(flag|f)\s*\(?(?<x>\d+)(,|\s+)(?<y>\d+)\)?").unwrap();
    let mut board = Board::new(args.get_rows(), args.get_cols(), args.get_mines());
    while board.ongoing() || !board.initialized() {
        println!("Enter coordinate to open (int,int): ");

        let mut coord = String::new();
        io::stdin()
            .read_line(&mut coord)
            .expect("Failed to read line");
        coord = coord.to_lowercase();

        let caps_flag = re_flag.captures(&coord);
        match caps_flag {
            Some(c) => {
                let (x, y) = (c.name("x").unwrap().as_str(), c.name("y").unwrap().as_str());
                let (x, y) = (x.trim().parse::<usize>(), y.trim().parse::<usize>());
                if x.is_err() || y.is_err() {
                    println!("Could not parse coordinates to usize, try again.");
                    continue;
                } else {
                    let flag_res = board.flag((x.unwrap(), y.unwrap()));
                    if let Err(e) = flag_res {
                        match e {
                            FlagError::AlreadyOpen => {
                                println!("This field is already open, try again.")
                            }
                            FlagError::OutOfBounds => {
                                println!("That coordinate set is out of bounds, try again")
                            }
                            FlagError::AlreadyWon => {
                                panic!("This game is already won.")
                            }
                            FlagError::MinesNotInit => {
                                panic!("Mines have not been initialized.")
                            }
                            FlagError::AlreadyLost => panic!("Game is already lost."),
                        }
                    }
                }
            }
            None => {
                let caps_open = re_open.captures(&coord);
                match caps_open {
                    None => {
                        println!("Invalid coordinate entered, try again.");
                        continue;
                    }
                    Some(c) => {
                        let (x, y) = (c.name("x").unwrap().as_str(), c.name("y").unwrap().as_str());
                        let (x, y) = (x.trim().parse::<usize>(), y.trim().parse::<usize>());
                        if x.is_err() || y.is_err() {
                            println!("Could not parse coordinates to usize, try again.");
                            continue;
                        } else {
                            let (x, y) = (x.unwrap(), y.unwrap());
                            match board.initialized() {
                                false => board.init_mines((x, y), args.get_seed()),
                                true => {
                                    let open_res = board.open((x, y));
                                    if let Err(e) = open_res {
                                        match e {
                                            OpenError::AlreadyOpen => {
                                                println!("This field is already open, try again.")
                                            }
                                            OpenError::AlreadyFlagged => {
                                                println!(
                                                    "This field is already flagged, try again."
                                                )
                                            }
                                            OpenError::OutOfBounds => {
                                                println!("That coordinate set is out of bounds, try again")
                                            }
                                            OpenError::AlreadyWon => {
                                                panic!("This game is already won.")
                                            }
                                            OpenError::MinesNotInit => {
                                                panic!("Mines have not been initialized.")
                                            }
                                            OpenError::AlreadyLost => {
                                                panic!("Game is already lost.")
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        println!("Current board: \n{board}");
    }
    if board.lost() {
        println!("You lost!")
    } else {
        println!("Congratulations, you won!")
    }
}
