//! Yew view layer: renders a `Board` and turns arrow keys into moves.
//! Every rule lives in `game`; this module only maps input to actions and
//! state to markup.

use crate::game::{Board, Direction, Outcome, SIZE};
use gloo::events::EventListener;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};
use yew::prelude::*;

/// Backgrounds for 2, 4, 8, ... 2048. Larger tiles reuse the last colour.
const PALETTE: [&str; 11] = [
    "#ebe0d5", "#ebddc2", "#f1a96f", "#f58b59", "#f67154", "#f65e3b", "#edcf72", "#edcc61",
    "#edc850", "#edc53f", "#edc22e",
];

pub enum Action {
    Shift(Direction),
    Reset,
}

impl Reducible for Board {
    type Action = Action;

    /// Randomness is confined here, at the edge of the pure engine.
    fn reduce(self: Rc<Self>, action: Action) -> Rc<Self> {
        let mut rng = rand::thread_rng();
        match action {
            Action::Reset => Rc::new(Board::new(&mut rng)),
            // A blocked move hands back an equal `Board`, which `use_reducer_eq`
            // compares with `PartialEq` and then skips the re-render.
            Action::Shift(direction) => self.step(direction, &mut rng).map(Rc::new).unwrap_or(self),
        }
    }
}

#[function_component]
pub fn App() -> Html {
    let board = use_reducer_eq(|| Board::new(&mut rand::thread_rng()));
    let outcome = board.outcome();
    use_arrow_keys(board.dispatcher(), outcome.is_none());
    let reset = {
        let board = board.clone();
        Callback::from(move |_: MouseEvent| board.dispatch(Action::Reset))
    };
    html! {
        <main>
            <header class="header">
                <h1>{"2048"}</h1>
                <p class="score">{format!("Score {}", board.score())}</p>
                <button onclick={reset}>{"New game"}</button>
            </header>
            <p class="status">{status(outcome)}</p>
            <div class="grid">{for board.cells().iter().map(row)}</div>
        </main>
    }
}

/// Listens on the document rather than a focusable element, so the player never
/// has to click the grid first. Torn down when the game ends.
#[hook]
fn use_arrow_keys(dispatcher: UseReducerDispatcher<Board>, playable: bool) {
    use_effect_with_deps(
        move |&playable| {
            let handler = move |event: &Event| dispatch_key(&dispatcher, event);
            let listener =
                playable.then(|| EventListener::new(&gloo::utils::document(), "keydown", handler));
            move || drop(listener)
        },
        playable,
    );
}

fn dispatch_key(dispatcher: &UseReducerDispatcher<Board>, event: &Event) {
    let direction = event
        .dyn_ref::<KeyboardEvent>()
        .and_then(|key| direction_of(&key.key()));
    if let Some(direction) = direction {
        dispatcher.dispatch(Action::Shift(direction));
    }
}

/// Browser key names are a view concern, so they stop here.
fn direction_of(key: &str) -> Option<Direction> {
    match key {
        "ArrowLeft" => Some(Direction::Left),
        "ArrowRight" => Some(Direction::Right),
        "ArrowUp" => Some(Direction::Up),
        "ArrowDown" => Some(Direction::Down),
        _ => None,
    }
}

fn row(cells: &[u32; SIZE]) -> Html {
    html! { <div class="row">{for cells.iter().map(|&value| tile(value))}</div> }
}

fn tile(value: u32) -> Html {
    if value == 0 {
        return html! { <div class="cell"></div> };
    }
    html! { <div class="cell tile" style={colour(value)}>{value}</div> }
}

/// Tier is log2(value) - 1, clamped to the palette.
fn colour(value: u32) -> String {
    let tier = (value.trailing_zeros() as usize).saturating_sub(1);
    format!("background-color: {}", PALETTE[tier.min(PALETTE.len() - 1)])
}

fn status(outcome: Option<Outcome>) -> &'static str {
    match outcome {
        Some(Outcome::Won) => "2048 reached. You win.",
        Some(Outcome::Lost) => "No moves left. Game over.",
        None => "Arrow keys to move.",
    }
}
