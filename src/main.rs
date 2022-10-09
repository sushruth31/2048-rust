use gloo::{console::log, events::EventListener};
use rand::Rng;
use wasm_bindgen::JsCast;
use yew::prelude::*;

const GRID_SIZE: i32 = 4;

type BoardType = Vec<Vec<i32>>;

fn shift_board_right(grid: &mut BoardType) -> BoardType {
    for (i, row) in grid.iter_mut().enumerate() {
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
                    row[next_j] = next * 2;
                    row[j as usize] = 0;
                }
            }
        }
    }
    grid.to_owned()
}

fn shift_board_left(grid: &mut BoardType) -> BoardType {
    for (i, row) in grid.iter_mut().enumerate() {
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
                    row[j as usize] = curr * 2;
                    row[next_i] = 0;
                }
            }
        }
    }
    grid.to_owned()
}

fn rotate_grid_left(grid: &mut BoardType) -> BoardType {
    let mut target = build_grid(GRID_SIZE);
    for (i, row) in grid.iter().enumerate() {
        for (j, num) in row.iter().enumerate() {
            //grid row should become target col
            //grid col should become target row but its gridsize -1 - col
            let targetrow = GRID_SIZE - 1 - (j as i32);
            target[targetrow as usize][i] = grid[i][j]
        }
    }
    target
}

fn rotate_grid_right(grid: &mut BoardType) -> BoardType {
    let mut target = build_grid(GRID_SIZE);
    for (i, row) in grid.iter().enumerate() {
        for (j, num) in row.iter().enumerate() {
            //grid row should be target col but Gridsize - it
            //grid col should become target row but its gridsize -1 - col
            let targetcol = GRID_SIZE - 1 - (i as i32);
            target[j][targetcol as usize] = grid[i][j];
        }
    }
    target
}

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

    fn shift_board_left(&mut self) -> Board {
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
                        row[j as usize] = curr * 2;
                        row[next_i] = 0;
                    }
                }
            }
        }
        Board {
            board: self.board.to_owned(),
        }
    }

    fn shift_board_right(&mut self) -> Board {
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
                        row[next_j] = next * 2;
                        row[j as usize] = 0;
                    }
                }
            }
        }
        Board {
            board: self.board.to_owned(),
        }
    }

    fn shift_down(&mut self) -> Board {
        //rotate right shift lef rotate left
        self.rotate_right().shift_board_left().rotate_left()
    }
}

#[function_component]
fn App() -> Html {
    let grid: UseStateHandle<BoardType> = use_state(|| build_grid(GRID_SIZE));
    {
        let grid = grid.clone();
        let grid_inner = grid.clone();
        let mut board = Board {
            board: grid.to_vec(),
        };
        use_effect_with_deps(
            move |_| {
                let document = gloo::utils::document();
                let listener = EventListener::new(&document, "keydown", move |evt| {
                    let e = evt.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
                    let key = e.key();
                    match key.as_str() {
                        "ArrowRight" => {
                            grid_inner.set(board.shift_board_right().board);
                        }
                        "ArrowLeft" => {
                            grid_inner.set(board.shift_board_left().board);
                        }
                        "ArrowUp" => {
                            grid_inner.set(board.shift_up().board);
                        }
                        "ArrowDown" => {
                            grid_inner.set(board.shift_down().board);
                        }
                        _ => (),
                    }
                });
                || drop(listener)
            },
            grid,
        );
    }
    let rows = grid.to_vec().into_iter().map(|row| {
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
    let rand_row2 = random_int(0, size, Some(rand_row1));
    let rand_col2 = random_int(0, size, Some(rand_col1));

    target[rand_row2 as usize][rand_col2 as usize] = 2;
    target
}

fn random_int(min: i32, max: i32, exclude: Option<i32>) -> i32 {
    let mut rng = rand::thread_rng();
    if exclude.is_none() {
        return rng.gen_range(min..max);
    }
    loop {
        let attempt = rng.gen_range(min..max);
        if let Some(exclude) = exclude {
            if exclude != attempt {
                return attempt;
            }
        }
    }
}
