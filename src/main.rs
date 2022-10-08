use yew::prelude::*;

const GRID_SIZE: u32 = 8;

#[derive(Properties, PartialEq)]
pub struct GridProps {
    render_cell: Callback<String, Html>,
}

#[function_component]
fn Grid(props: &GridProps) -> Html {
    let rows = (0..GRID_SIZE).map(|i| {
        let cols = (0..GRID_SIZE).map(|j| {
            let mut key = "".to_string();
            let combined = format!("{}{}{}", i.to_string(), "+".to_string(), j.to_string());
            key.push_str(&combined);

            html! {
                <div class="col">
                {props.render_cell.emit(key)}
                </div>
            }
        });
        html! {
            <div class="row">
            {for cols}
            </div>
        }
    });

    html! {
        <>
        {for rows}
        </>
    }
}

#[function_component]
fn App() -> Html {
    let render_cell = Callback::from(move |key: String| {
        html! {
            <div>{key}</div>
        }
    });

    html! {
        <>
            <h1>{"Welcome to 2048"}</h1>
            <Grid {render_cell} />
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
