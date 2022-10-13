use gloo::{console::log, events::EventListener};
use rand::Rng;
use std::{collections::HashMap, ops::Deref};
use wasm_bindgen::JsCast;
use yew::prelude::*;

const GRID_SIZE: i32 = 4;

type BoardType = Vec<Vec<i32>>;

#[derive(Clone, PartialEq, Debug, Properties)]
pub struct Board {
    board: BoardType,
}

impl Board {
    fn rotate_left(&mut self) -> Board {
        let mut target = build_grid(GRID_SIZE);
        for (i, row) in self.board.iter().enumerate() {
            for (j, num) in row.iter().enumerate() {
                //grid row should become target col
                //grid col should become target row but its gridsize -1 - col
                let targetrow = GRID_SIZE - 1 - (j as i32);
                target[targetrow as usize][i] = self.board[i][j]
            }
        }
        Board { board: target }
    }

    fn shift_up(&mut self) -> Board {
        //rotate left then shift left the rotate right
        self.rotate_left().shift_board_left().rotate_right()
    }

    fn rotate_right(&mut self) -> Board {
        let mut target = build_grid(GRID_SIZE);
        for (i, row) in self.board.iter().enumerate() {
            for (j, num) in row.iter().enumerate() {
                //grid row should be target col but Gridsize - it
                //grid col should become target row but its gridsize -1 - col
                let targetcol = GRID_SIZE - 1 - (i as i32);
                target[j][targetcol as usize] = self.board[i][j];
            }
        }
        Board { board: target }
    }

    fn num_zeros(&self) -> i32 {
        (self
            .get_board()
            .into_iter()
            .flatten()
            .filter(|n| n == &0)
            .collect::<Vec<i32>>()
            .len()) as i32
    }

    fn shift_board_left(&mut self) -> Board {
        let copy = self.clone();
        for (i, row) in self.board.iter_mut().enumerate() {
            let mut count = 0;
            while count < 5 {
                count += 1;
                for j in 0..GRID_SIZE - 1 {
                    let curr = row[j as usize];
                    let next_i = (j + 1) as usize;
                    let next = row[next_i];
                    if curr == next {
                        //add if numbers are the same
                        row[j as usize] = curr * 2;
                        row[next_i] = 0;
                    } else if curr == 0 {
                        row[j as usize] = next;
                        row[next_i] = 0;
                    }

                    //move if value was not a modified one.
                }
            }
        }
        //todo only add if there has been a change in board pieces
        if !self.boards_eq(&copy.board[..]) {
            self.add_num_to_board();
        }

        Board {
            board: self.board.to_owned(),
        }
    }

    fn boards_eq(&self, compare_board: &[Vec<i32>]) -> bool {
        for (i, row) in self.get_board().into_iter().enumerate() {
            for (j, num) in row.into_iter().enumerate() {
                if compare_board[i][j] != num {
                    return false;
                }
            }
        }
        true
    }

    fn shift_board_right(&mut self) -> Board {
        let copy = self.clone();
        for (i, row) in self.board.iter_mut().enumerate() {
            let mut count = 0;
            while count < 5 {
                count += 1;
                for j in (0..GRID_SIZE - 1).rev() {
                    let next_j = (j + 1) as usize;
                    let left = row[j as usize];
                    let right = row[next_j];

                    if left == right {
                        //right * 2
                        row[next_j] = right * 2;
                        row[j as usize] = 0;
                    } else if right == 0 {
                        //shift right
                        row[next_j] = left;
                        row[j as usize] = 0;
                    }
                }
            }
        }

        if !self.boards_eq(&copy.board[..]) {
            self.add_num_to_board();
        }

        Board {
            board: self.board.to_owned(),
        }
    }

    fn shift_down(&mut self) -> Board {
        //rotate right shift lef rotaje left
        self.rotate_right().shift_board_left().rotate_left()
    }

    fn new(size: i32) -> Board {
        Board {
            board: build_grid(size),
        }
    }

    fn get_board(&self) -> BoardType {
        self.board.to_vec()
    }

    fn add_num_to_board(&mut self) {
        //cannot add where num > 0
        let coord = self.find_possible_coord();
        if let Some(coord) = coord {
            //add only if we have at least one coord
            self.add_val_to_board(coord, 2);
        }
    }

    fn is_board_full(&self) -> bool {
        self.num_zeros() == 0
    }

    fn find_possible_coord(&self) -> Option<(i32, i32)> {
        //check to make sure board is not fullj
        //TODO check edge cases
        if self.is_board_full() {
            return None;
        }
        loop {
            let rowattempt = random_int(0, GRID_SIZE, None);
            let colattempt = random_int(0, GRID_SIZE, None);
            if self.board[rowattempt as usize][colattempt as usize] == 0 {
                return Some((rowattempt, colattempt));
            }
        }
    }

    fn add_val_to_board(&mut self, coord: (i32, i32), val: i32) -> () {
        self.board[coord.0 as usize][coord.1 as usize] = val;
    }

    fn is_game_over(&self) -> Option<Result> {
        //win if one number == 2048
        if self.get_board().into_iter().flatten().any(|v| v == 2048) {
            return Some(Result::WIN);
        }
        if self.num_zeros() == 0 {
            return Some(Result::LOSE);
        }

        None
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Result {
    WIN,
    LOSE,
}

#[function_component]
fn App() -> Html {
    let board = use_state(|| Board::new(GRID_SIZE));
    let colormap = use_memo(|_| get_color_map(), ());
    let outcome: UseStateHandle<Option<Result>> = use_state(|| None);
    let outcomeval = outcome.deref();
    {
        let board = board.clone();
        let outcome = outcome.clone();
        use_effect_with_deps(
            move |(board, outcome)| {
                //check if win or lose here
                let mut gameover = false;
                if let Some(goopt) = board.is_game_over() {
                    gameover = true;
                    outcome.set(Some(goopt))
                }
                let document = gloo::utils::document();
                let board = board.clone();
                let listener = EventListener::new(&document, "keydown", move |evt| {
                    if gameover {
                        return;
                    }
                    let e = evt.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
                    let key = e.key();
                    match key.as_str() {
                        "ArrowRight" => {
                            let newboard = board.deref().to_owned().shift_board_right();
                            board.set(newboard);
                        }
                        "ArrowLeft" => {
                            board.set(board.deref().to_owned().shift_board_left());
                        }
                        "ArrowUp" => {
                            board.set(board.deref().to_owned().shift_up());
                        }
                        "ArrowDown" => {
                            board.set(board.deref().to_owned().shift_down());
                        }
                        _ => (),
                    }
                    //add the next squares.
                });
                || drop(listener)
            },
            (board, outcome),
        );
    }
    let rows = board.get_board().into_iter().map(|row| {
        let cols = row.into_iter().map(|num| {
            let mut class = "flexbox".to_string();
            if let Some(level) = (*colormap).get(&num) {
                class.push_str(&format!(" {}", level.to_string()));
            }
            html! {
                <div class="col">
                if num == 0 {
                   <div></div>
                } else {
                    <div {class}>{num}</div>
                }
                </div>
            }
        });
        html! {
            <>
            <div class="row">{for cols}</div>
            </>
        }
    });

    let onreset = Callback::from(move |_: MouseEvent| {
        let board = board.clone();
        board.set(Board::new(GRID_SIZE));
    });
    html! {
        <>
        <div class="flexbox">
        <h1>{"Welcome to 2048"}</h1>
        <button onclick={onreset}>
        {"New Game"}
        </button>
        </div>
            if let Some(outcome) = outcomeval {
                if outcome == &Result::WIN {
                    <div>{"You Win"}</div>
                } else {
                    <div>{"You Win"}</div>
                }
            }
        <div class="container">
        {for rows}
        </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

fn to_tuple(key: &str) -> (i32, i32) {
    let k: Vec<i32> = key
        .split("+")
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|num| num.parse::<i32>().unwrap())
        .collect();

    (k[0], k[1])
}

fn zeros(len: i32) -> Vec<i32> {
    vec![0; len as usize]
}

fn build_grid(size: i32) -> Vec<Vec<i32>> {
    let rand_row1 = random_int(0, size, None);
    let rand_col1 = random_int(0, size, None);
    let mut target: Vec<Vec<i32>> = vec![];
    for _ in 0..size {
        target.push(zeros(size));
    }
    target[rand_row1 as usize][rand_col1 as usize] = 2;
    let rand_row2 = random_int(0, size, Some(&vec![rand_row1]));
    let rand_col2 = random_int(0, size, Some(&vec![rand_col1]));

    target[rand_row2 as usize][rand_col2 as usize] = 2;
    target
}

fn random_int(min: i32, max: i32, exclude: Option<&Vec<i32>>) -> i32 {
    let mut rng = rand::thread_rng();
    if exclude.is_none() {
        return rng.gen_range(min..max);
    }
    loop {
        let attempt = rng.gen_range(min..max);
        if !exclude.clone().unwrap().contains(&attempt) {
            return attempt;
        }
    }
}

fn make_range(min: i32, max: i32, interval: i32) -> Vec<i32> {
    let mut tar: Vec<i32> = Vec::new();
    let mut last = min;
    loop {
        let new = if last == 0 { interval } else { last * interval };
        if new >= max {
            break;
        }
        tar.push(new);
        last = new;
    }
    tar
}

//8 -> 3. 3 used for classname
fn get_color_map() -> HashMap<i32, String> {
    let mut levelmap = HashMap::new();
    levelmap.insert(1, "one");
    levelmap.insert(2, "two");
    levelmap.insert(3, "three");
    levelmap.insert(4, "four");
    levelmap.insert(5, "five");
    let mut hashmap = HashMap::new();
    for (i, val) in make_range(0, 2048, 2).into_iter().enumerate() {
        let i = (i + 1) as i32;
        let levelstring = levelmap.get(&i);
        if let Some(levelstring) = levelstring {
            hashmap.insert(val, levelstring.to_string());
        }
    }

    hashmap
}
