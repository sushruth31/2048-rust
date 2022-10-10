use gloo::{console::log, events::EventListener};
use rand::Rng;
use std::{borrow::BorrowMut, ops::Deref};
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
        let mut count = 0;

        for row in self.get_board().iter() {
            for num in row.to_vec().into_iter() {
                if num == 0 {
                    count += 1;
                }
            }
        }

        count
    }

    fn shift_board_left(&mut self) -> Board {
        let ogboard = self.clone();
        for (i, row) in self.board.iter_mut().enumerate() {
            let mut count = 0;
            while count < 5 {
                count += 1;
                for j in 0..GRID_SIZE - 1 {
                    let curr = row[j as usize];
                    let next_i = (j + 1) as usize;
                    let next = row[next_i];
                    if curr == 0 {
                        //move next left;
                        row[j as usize] = next;
                        row[next_i] = 0;
                    }
                    if curr == next {
                        //add if numbers are the same
                        row[j as usize] = curr + next;
                        row[next_i] = 0;
                    }
                }
            }
        }
        let mut board = self.clone();
        if ogboard.num_zeros() != self.num_zeros() {
            board.add_nums_to_board();
        }

        Board { board: board.board }
    }

    fn shift_board_right(&mut self) -> Board {
        let ogboard = self.clone();
        for (i, row) in self.board.iter_mut().enumerate() {
            let mut count = 0;
            while count < 5 {
                count += 1;
                for j in 0..GRID_SIZE - 1 {
                    let curr = row[j as usize];
                    let next_j = (j + 1) as usize;
                    let next = row[next_j];
                    if next == 0 {
                        //shift right
                        row[next_j] = curr;
                        row[j as usize] = 0;
                    }
                    if curr == next {
                        row[next_j] = next + curr;
                        row[j as usize] = 0;
                    }
                }
            }
        }
        let mut board = self.clone();
        if ogboard.num_zeros() != self.num_zeros() {
            board.add_nums_to_board();
        }
        Board { board: board.board }
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

    fn add_nums_to_board(&mut self) {
        //cannot add where num > 0
        //TODO check edge cases
        let coord1 = self.find_possible_coord();
        let coord2 = self.find_possible_coord();
        if let Some(coord1) = coord1 {
            if let Some(coord2) = coord2 {
                self.add_val_to_board(coord1, 2);
                self.add_val_to_board(coord2, 2);
            }
        }
    }

    fn is_board_full(&self) -> bool {
        false
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
            if self.get_board()[rowattempt as usize][colattempt as usize] == 0 {
                return Some((rowattempt, colattempt));
            }
        }
    }

    fn add_val_to_board(&mut self, coord: (i32, i32), val: i32) -> Self {
        for (i, row) in self.get_board().iter_mut().enumerate() {
            for (j, num) in row.iter().enumerate() {
                if i as i32 == coord.0 && j as i32 == coord.1 {
                    self.board[i][j] = val;
                }
            }
        }

        Board {
            board: self.board.to_owned(),
        }
    }
}

#[function_component]
fn App() -> Html {
    let board: UseStateHandle<Board> = use_state(|| Board::new(GRID_SIZE));
    {
        let board = board.clone();
        use_effect_with_deps(
            move |board| {
                let document = gloo::utils::document();
                let board = board.clone();
                let listener = EventListener::new(&document, "keydown", move |evt| {
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
            board,
        );
    }
    let rows = board.get_board().into_iter().map(|row| {
        let cols = row.into_iter().map(|num| {
            html! {
                <div class="col">
                if num == 0 {
                   <div></div>
                } else {
                    {num}
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
    html! {
        <>
        <h1>{"Welcome to 2048"}</h1>
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

fn multple_random_ints(min: i32, max: i32, count: i32) -> Vec<i32> {
    let mut tar: Vec<i32> = Vec::with_capacity(count as usize);
    for _ in 0..(count as usize) {
        tar.push(random_int(min, max, Some(&tar)))
    }
    tar
}
