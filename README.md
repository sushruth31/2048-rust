# 2048 — the game in Rust, compiled to WebAssembly

A playable 2048 built as a weekend exercise in Rust on the front end. The whole
rule set is a pure, dependency-free module that the browser layer only renders;
the interesting part is getting the merge semantics exactly right (a merged tile
must not merge again in the same move) and deciding that the game is lost only
when *no* direction moves a tile.

![ci](https://github.com/sushruth31/2048-rust/actions/workflows/ci.yml/badge.svg)

## Stack

- **Rust 2021** — rules engine, no dependencies beyond `rand`.
- **Yew** — component model in Rust; pinned to a `master` revision rather than
  the 0.19.3 release, because that release predates `#[hook]` and
  name-inferring `#[function_component]`. `Cargo.lock` pins the exact revision.
- **Trunk** — wasm bundler; it builds the crate, runs `wasm-bindgen`, hashes the
  assets and serves them.
- **`getrandom` with the `js` feature** — otherwise `rand` has no entropy source
  under `wasm32-unknown-unknown`.

## Running it

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open          # http://127.0.0.1:8080
cargo test                  # rules engine, host toolchain, no wasm needed
```

There is no configuration and no secrets: the game is entirely client-side and
talks to nothing, so there is no `.env` to fill in.

## Architecture

```
index.html ──trunk──> dist/          static shell + hashed wasm/js/css
      │
src/main.rs        entry point; wasm mounts <App>, host build is a stub
      │
src/lib.rs ──┬── game.rs            pure rules: Board, Direction, Outcome
             │                      no browser APIs, RNG passed in
             └── ui.rs   (wasm32)   Yew component, keyboard -> Action,
                                    Board -> Html, palette
```

| Module    | Responsibility                                     | Target      |
| --------- | -------------------------------------------------- | ----------- |
| `game`    | slide, merge, spawn, score, win/loss                | all         |
| `ui`      | reducer, key listener, rendering                   | wasm32 only |
| `main`    | mounts the app                                     | all         |

## Design notes

- **One slide routine covers four directions.** `Direction::cell(line, step)`
  maps a logical position — line *l*, *step* cells in from the edge tiles move
  toward — onto a grid coordinate, so left/right/up/down all reduce to "collapse
  a line toward index 0". That replaced a rotate-left/shift/rotate-right dance
  that allocated two extra grids on every vertical move. A move now touches each
  of the 16 cells twice — once read, once written: O(n²) for an n×n board, with
  no allocation in the slide itself.
- **Merging is `Peekable::next_if_eq`.** Filter out the zeros, then consume a
  tile and conditionally consume its equal neighbour. That gives the rule "each
  tile merges at most once per move" structurally rather than with a
  `already_merged` flag: `[2,2,2,2] → [4,4,0,0]`, never `[8,0,0,0]`, and
  `[2,2,4] → [4,4]`, never `[8]`. The obvious implementation — repeat a
  pass-and-merge sweep until the row stops changing — gets both of those wrong,
  which is what the original version of this repo did.
- **Loss means "no legal move", not "grid is full".** A full grid holding an
  adjacent equal pair is still playable. `outcome()` therefore shifts a copy in
  all four directions and reports `Lost` only if none of them changes a cell:
  four throwaway shifts, ~64 cell reads, run once per render — cheap enough that
  maintaining an incremental adjacency index would have been a worse trade.
- **Randomness is a parameter, not an ambient effect.** `Board::new` and
  `Board::step` take `&mut impl Rng`, so tests drive them with a seeded
  `StdRng` and assert on exact boards; `thread_rng()` appears only in the view
  layer, in the reducer and its initialiser, and nowhere in `game`. `step` also
  returns `Option<Board>` and yields `None` for a blocked move, which is what
  stops the player being handed a free tile for pressing a direction that does
  nothing.
- **Browser dependencies are target-gated.** `yew`, `gloo`, `web-sys` and
  `wasm-bindgen` sit under `[target.'cfg(target_arch = "wasm32")'.dependencies]`
  and `ui` is `#[cfg(target_arch = "wasm32")]`, so `cargo test` compiles the
  rules against 7 crates instead of the 75 the wasm build pulls in. CI lints
  both targets so the view layer stays honest.
- **A blocked move renders nothing.** The reducer hands back an unchanged
  `Board` when `step` returns `None`, and the component uses `use_reducer_eq`,
  not `use_reducer` — the former passes `Board::ne` as the re-render predicate,
  the latter hardcodes `true`. So an arrow key held against a wall costs one
  68-byte `PartialEq` and no virtual-DOM diff.

## Tests

`cargo test` — 15 unit tests over the rules engine, covering the cases a naive
implementation gets wrong:

- a full row merges into two tiles, never one (`[2,2,2,2] → [4,4,0,0]`)
- a tile that just merged cannot merge again this move (`[2,2,4] → [4,4]`)
- the pair nearest the destination edge merges first (`[2,2,2] →` right `→ [2,4]`)
- score credits the value of the tile created, not the tiles consumed
- a blocked direction leaves the board untouched and spawns nothing
- a full board with an adjacent equal pair is not a loss; a checkerboard is
- spawns land only on empty cells, and only ever 2 or 4
