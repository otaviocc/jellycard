# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`jellycard` is a single Rust binary that renders a Jellyfin library card PNG
from a library name.

## Commands

```sh
make build                        # cargo build --release
make test                         # cargo test
make lint                         # cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
make install                      # cargo install --path . --locked --force

cargo test render::               # the renderer's unit tests
```

`fmt`, `lint` and `test` must all pass before a commit.

## Architecture

`src/main.rs` is the whole control flow: it reads either a single argument or
stdin (`-`, one name per non-empty line), slugs each name into a filename, and
writes the PNG into the current directory.

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
wide for the 10% side margins.

This replaces a Pillow script (`generate_card.py` in the `jellyfin-library-cards`
skill), and the geometry is a port of it, so cards generated before and after
line up:

- **No shaping.** Glyphs are positioned from `hmtx` advances alone. Fredoka
  keeps its kerning in `GPOS` and has no `kern` table, so Pillow's basic layout
  applied none of it; running swash's shaper would set new cards tighter than
  every card already on disk.
- **Metric-based centring**, Pillow's `"mm"` anchor: the advance width and the
  ascent/descent box are what get centred, not the ink, so every card shares one
  baseline whatever letters its name uses.
- Rasterization is swash rather than FreeType, so output is *visually* identical
  but not byte-identical — ink lands within a pixel or two of the Python script.
  `fit_font_size` does reproduce its sizes exactly (`Collections` → 283,
  `Documentaries` → 210), and a change there is a regression, not a rounding
  difference.

## Tests

Unit tests live in a `mod tests` at the bottom of every source file. The
renderer's tests pin the shared font size, the transparent canvas and the
gradient endpoints; the fastest visual check against a reference is still to run
the old Pillow script and compare alpha bounding boxes.

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
- `README.md` is a **product page** for someone using the program, with no
  architecture section. A change to the CLI or the card style updates it in the
  same commit; a change to how the code works does not touch it.
