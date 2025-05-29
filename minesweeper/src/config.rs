use clap::Parser;

/// Generate minesweeper boards
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Seed for the board generation
    #[arg(short, long, default_value=None)]
    seed: Option<u64>,

    /// Number of board rows
    #[arg(short, long, default_value = "9")]
    rows: usize,

    /// Number of board cols
    #[arg(short, long, default_value = "9")]
    cols: usize,

    /// Number of mines
    #[arg(short, long, default_value = "10")]
    mines: usize,
}

impl Args {
    pub fn get_seed(&self) -> Option<u64> {
        self.seed
    }
    pub fn get_rows(&self) -> usize {
        self.rows
    }
    pub fn get_cols(&self) -> usize {
        self.cols
    }
    pub fn get_mines(&self) -> usize {
        self.mines
    }
}
