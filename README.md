# LazyConfig

LazyConfig is a terminal dashboard for ChezMoi-managed application
configurations.

## Setup

Run `lazyconfig init`. It creates `~/.config/lazyconfig/config.toml` from
`examples/config.toml` and prints the `chezmoi add` command for it. LazyConfig
does not run `chezmoi add` and does not start until ChezMoi manages the
registry.

The `adapter` of a registry entry sets what LazyConfig can edit:

- `raw`: no fields. Use `e` to edit the source.
- `kitty`: `font_size`, `background_opacity`, and the theme `include` line of
  `kitty.conf`.
- `starship`: `add_newline`.

Only targets already managed by ChezMoi are selectable. LazyConfig resolves the
source path for each target at runtime.

## Run

```sh
cargo run
```

## Keys

- `j` or Down, `k` or Up: move in the focused pane.
- `Tab`: switch between the config list and the field list.
- `+` and `-`: change the selected field. Nothing is written.
- `p`: show the changed source lines.
- `w`: write the change to the ChezMoi source, then show the ChezMoi diff.
  LazyConfig does not write if the source file changed after it was read.
- `Esc`: discard the unsaved change, or close the diff.
- `a`: show the ChezMoi diff for the selected target. Press `a` again to apply.
- `e`: open the selected ChezMoi source path in `$EDITOR`; uses `nvim` when
  `$EDITOR` is unset.
- `d`: scan top-level `~/.config` entries with `fff-search`; add only new,
  ChezMoi-managed applications to LazyConfig's ChezMoi source registry.
- `r`: refresh registry state from ChezMoi.
- `q`: quit.
