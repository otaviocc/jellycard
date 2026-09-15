# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`jellycard` is a single Rust binary that renders a Jellyfin library card PNG
from a library name.

## Commands

```sh
make build                        # cargo build --release
make test                         # cargo test
make lint                         # cargo clippy --all-targets -- -D warnings
make run                          # cargo run --release -- "4K Movies"
cargo fmt --all -- --check
make install                      # cargo install --path . --locked --force
make uninstall                    # cargo uninstall jellycard

cargo test render::               # the renderer's unit tests
```

`fmt`, `lint` and `test` must all pass before a commit.

## Layout

```
src/main.rs        CLI, slug, PNG encoding
src/render.rs      everything about the card image
assets/Fredoka.ttf embedded font, with its OFL licence beside it
.github/workflows  CI on push and PR, release on a `v*.*.*` tag
```

Dependencies are `png` and `swash`, nothing else.

## Architecture

`src/main.rs` is the whole control flow: it reads either a single argument or
stdin (`-`, one name per non-empty line), slugs each name into a filename, and
writes the PNG into the current directory. A name that slugs to nothing is an
error; any failure exits non-zero, and bad usage exits 2.

```
name → render::fit_font_size    binary search, largest size <= 300 that fits
     → render::text_mask        per-glyph coverage composited into one canvas mask
     → render::ink_bbox         the gradient's span
     → render::render           RGBA, straight alpha
     → main::write_png
```

The style is deliberately not configurable: a new card has to match the ones
already in the Jellyfin library. Fredoka at variable weight 600 (embedded via
`include_bytes!`), 1800×1000 transparent canvas, `#AA5CC3` → `#00A4DC` spanning
the text's ink box, size 300 for every card and shrunk only when a name is too
wide for the 10% side margins or taller than 60% of the canvas.

Two geometry decisions keep new cards lining up with the ones already on disk,
and both look like omissions if you meet them cold:

- **No shaping.** Glyphs are positioned from `hmtx` advances alone. Fredoka
  keeps its kerning in `GPOS` and has no `kern` table, so none of it applies;
  running swash's shaper instead would set new cards tighter than every
  existing card.
- **Metric-based centring.** The advance width and the ascent/descent box are
  what get centred, not the ink, so every card shares one baseline whatever
  letters its name uses.

`fit_font_size` is the part with exact expected values (`Collections` → 283,
`Documentaries` → 210); a change there is a regression, not a rounding
difference.

## Tests

Unit tests live in a `mod tests` at the bottom of every source file. The
renderer's tests pin the shared font size, the transparent canvas and the
gradient endpoints. Rasterization is not byte-pinned, so for a visual change
compare the alpha bounding box of a freshly generated card against a card
already in the library.

## Code conventions

- Rust edition 2024, `rustfmt.toml` as committed (`max_width = 130`,
  `use_small_heuristics = "Max"`).
- No `unsafe`. No `unwrap` outside tests; `expect` only with an invariant
  message.
- **No comments in Rust.** A file carries the two-line SPDX/copyright header
  and may carry a single `//!` line saying what it is, for navigation. Nothing
  else: no `///`, no `//`. A comment is a claim nobody checks, and it lends
  authority to whatever it sits above. Put the explanation in the commit message
  and PR body, which are dated and tied to a diff. If code needs a paragraph to
  be understood, prefer a name, a smaller function, or a test. (TOML in the repo
  is commented; the rule is about code.)
- `clippy -D warnings` makes `dead_code` a build failure, so every addition has
  to be constructed by the change that adds it.
- Add a dependency only when it earns its place, and say why in the commit body.

## Workflow

- Conventional Commits with a short imperative subject and a body explaining
  *why*. Stage paths explicitly — `git add -A` sweeps unrelated edits in.
- A release is a `v*.*.*` tag whose version matches `Cargo.toml`; the tag
  drives `release.yml`, which builds the four archives, publishes to crates.io
  (`CARGO_REGISTRY_TOKEN`) and creates the GitHub release. Run the workflow by
  hand first to rehearse everything up to the builds — publishing is one-way.
  `rust-version` in `Cargo.toml` is what the `msrv` CI job checks against.
- `README.md` is a **product page** for someone using the program, with no
  architecture section. A change to the CLI or the card style updates it in the
  same commit; a change to how the code works does not touch it.
