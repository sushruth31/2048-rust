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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn board(cells: [Line; SIZE]) -> Board {
        Board { cells, score: 0 }
    }

    fn rng() -> StdRng {
        StdRng::seed_from_u64(2048)
    }

    fn tiles(board: &Board) -> Vec<u32> {
        board
            .cells
            .iter()
            .flatten()
            .copied()
            .filter(|&t| t != 0)
            .collect()
    }

    #[test]
    fn slides_tiles_to_the_edge_closing_every_gap() {
        let shifted =
            board([[0, 0, 0, 2], [4, 0, 8, 0], [0; 4], [2, 0, 0, 4]]).shift(Direction::Left);
        assert_eq!(
            shifted.cells,
            [[2, 0, 0, 0], [4, 8, 0, 0], [0; 4], [2, 4, 0, 0]]
        );
    }

    #[test]
    fn merges_an_equal_pair_into_its_double() {
        let shifted = board([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]).shift(Direction::Left);
        assert_eq!(shifted.cells[0], [4, 0, 0, 0]);
    }

    #[test]
    fn merges_a_full_row_into_two_tiles_never_one() {
        let shifted = board([[2, 2, 2, 2], [0; 4], [0; 4], [0; 4]]).shift(Direction::Left);
        assert_eq!(shifted.cells[0], [4, 4, 0, 0]);
    }

    #[test]
    fn never_merges_a_tile_that_already_merged_this_move() {
        let shifted = board([[2, 2, 4, 0], [0; 4], [0; 4], [0; 4]]).shift(Direction::Left);
        assert_eq!(shifted.cells[0], [4, 4, 0, 0]);
    }

    #[test]
    fn merges_the_pair_nearest_the_destination_edge_first() {
        let shifted = board([[2, 2, 2, 0], [0; 4], [0; 4], [0; 4]]).shift(Direction::Right);
        assert_eq!(shifted.cells[0], [0, 0, 2, 4]);
    }

    #[test]
    fn scores_the_value_of_each_tile_created_by_a_merge() {
        let shifted = board([[4, 4, 8, 8], [2, 2, 0, 0], [0; 4], [0; 4]]).shift(Direction::Left);
        assert_eq!(shifted.score, 8 + 16 + 4);
    }

    #[test]
    fn moves_columns_up_and_down_independently() {
        let start = board([[2, 0, 0, 0], [2, 4, 0, 0], [0, 4, 0, 0], [0, 0, 2, 0]]);
        assert_eq!(start.shift(Direction::Up).cells[0], [4, 8, 2, 0]);
        assert_eq!(start.shift(Direction::Down).cells[3], [4, 8, 2, 0]);
    }

    #[test]
    fn leaves_the_board_untouched_when_a_direction_is_blocked() {
        let blocked = board([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        assert_eq!(blocked.shift(Direction::Left).cells, blocked.cells);
    }

    #[test]
    fn refuses_to_spawn_a_tile_on_a_move_that_changes_nothing() {
        let blocked = board([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        assert_eq!(blocked.step(Direction::Left, &mut rng()), None);
    }

    #[test]
    fn spawns_exactly_one_tile_per_move_that_changes_the_board() {
        let start = board([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
        let next = start
            .step(Direction::Left, &mut rng())
            .expect("row can merge");
        assert_eq!(tiles(&next).len(), 2);
        assert_eq!(next.cells[0][0], 4);
    }

    #[test]
    fn opens_the_game_with_two_tiles_worth_two_or_four() {
        let opening = tiles(&Board::new(&mut rng()));
        assert_eq!(opening.len(), 2);
        assert!(opening.iter().all(|&tile| tile == 2 || tile == 4));
    }

    #[test]
    fn keeps_a_full_board_alive_while_an_adjacent_pair_remains() {
        let full = board([[2, 2, 4, 8], [4, 8, 16, 32], [2, 4, 8, 16], [4, 8, 16, 32]]);
        assert_eq!(full.outcome(), None);
    }

    #[test]
    fn declares_a_loss_only_when_no_direction_moves_a_tile() {
        let dead = board([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        assert_eq!(dead.outcome(), Some(Outcome::Lost));
    }

    #[test]
    fn declares_a_win_when_the_winning_tile_appears() {
        let won = board([[WINNING_TILE, 0, 0, 0], [0; 4], [0; 4], [0; 4]]);
        assert_eq!(won.outcome(), Some(Outcome::Won));
    }

    #[test]
    fn fills_the_grid_without_ever_overwriting_a_tile() {
        let mut rng = rng();
        let mut filled = Board::default();
        for expected in 1..=SIZE * SIZE {
            filled.spawn(&mut rng);
            assert_eq!(tiles(&filled).len(), expected);
        }
        filled.spawn(&mut rng);
        assert_eq!(tiles(&filled).len(), SIZE * SIZE);
    }
}
