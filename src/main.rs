use gloo::console::log;
use rand::Rng;
use yew::prelude::*;

const GRID_SIZE: i32 = 4;

#[function_component]
fn App() -> Html {
    let grid: UseStateHandle<Vec<Vec<i32>>> = use_state(|| build_grid(GRID_SIZE));
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
