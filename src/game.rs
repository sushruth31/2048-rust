//! The 2048 rules engine. No rendering, no browser APIs, no global state:
//! every move is a pure function of a board and an injected random source.

use rand::seq::SliceRandom;
use rand::Rng;

pub const SIZE: usize = 4;
pub const WINNING_TILE: u32 = 2048;

/// Real 2048 spawns a 4 one time in ten; everything else is a 2.
const FOUR_SPAWN_CHANCE: f64 = 0.1;

type Line = [u32; SIZE];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ];

    /// Cell reached after `step` moves inward along `line`, counting from the
    /// edge tiles slide toward. Lets one left-slide routine serve all four moves.
    fn cell(self, line: usize, step: usize) -> (usize, usize) {
        let far = SIZE - 1 - step;
        match self {
            Direction::Left => (line, step),
            Direction::Right => (line, far),
            Direction::Up => (step, line),
            Direction::Down => (far, line),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Won,
    Lost,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Board {
    cells: [Line; SIZE],
    score: u32,
}

impl Board {
    /// A fresh game: empty grid plus the two opening tiles.
    pub fn new<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let mut board = Board::default();
        board.spawn(rng);
        board.spawn(rng);
        board
    }

    pub fn cells(&self) -> &[Line; SIZE] {
        &self.cells
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    /// One turn: slide, then spawn. Returns `None` when nothing moved, because
    /// a move that changes no tile must not hand the player a free tile.
    pub fn step<R: Rng + ?Sized>(&self, direction: Direction, rng: &mut R) -> Option<Self> {
        let mut next = self.shift(direction);
        if next.cells == self.cells {
            return None;
        }
        next.spawn(rng);
        Some(next)
    }

    /// Slides and merges every line toward `direction`. Deterministic.
    pub fn shift(&self, direction: Direction) -> Self {
        let mut next = *self;
        for line in 0..SIZE {
            let source = core::array::from_fn(|step| self.tile(direction.cell(line, step)));
            let (collapsed, gained) = collapse(source);
            for (step, tile) in collapsed.into_iter().enumerate() {
                let (row, col) = direction.cell(line, step);
                next.cells[row][col] = tile;
            }
            next.score += gained;
        }
        next
    }

    /// `Lost` only when no direction changes the board. A full grid still
    /// holding an adjacent equal pair is playable.
    pub fn outcome(&self) -> Option<Outcome> {
        if self
            .cells
            .iter()
            .flatten()
            .any(|&tile| tile >= WINNING_TILE)
        {
            return Some(Outcome::Won);
        }
        Direction::ALL
            .iter()
            .all(|&direction| self.shift(direction).cells == self.cells)
            .then_some(Outcome::Lost)
    }

    fn tile(&self, (row, col): (usize, usize)) -> u32 {
        self.cells[row][col]
    }

    /// Places a tile on a uniformly chosen empty cell; no-op on a full grid.
    fn spawn<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let empty: Vec<(usize, usize)> = (0..SIZE)
            .flat_map(|row| (0..SIZE).map(move |col| (row, col)))
            .filter(|&cell| self.tile(cell) == 0)
            .collect();
        if let Some(&(row, col)) = empty.choose(rng) {
            self.cells[row][col] = if rng.gen_bool(FOUR_SPAWN_CHANCE) {
                4
            } else {
                2
            };
        }
    }
}

/// Slides non-empty tiles to the front and merges each pair at most once,
/// reporting the score gained. O(SIZE) per line, single pass, no allocation.
fn collapse(line: Line) -> (Line, u32) {
    let mut tiles = line.into_iter().filter(|&tile| tile != 0).peekable();
    let (mut collapsed, mut gained, mut slot) = ([0; SIZE], 0, 0);
    while let Some(tile) = tiles.next() {
        let merged = tiles.next_if_eq(&tile).is_some();
        collapsed[slot] = if merged { tile * 2 } else { tile };
        gained += if merged { tile * 2 } else { 0 };
        slot += 1;
    }
    (collapsed, gained)
}
