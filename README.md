# LazyConfig

LazyConfig is a terminal dashboard for ChezMoi-managed application
configurations.

## Setup

Create and track `~/.config/lazyconfig/config.toml` with ChezMoi. Use
`examples/config.toml` as the starting registry.

Only targets already managed by ChezMoi are selectable. LazyConfig resolves the
source path for each target at runtime.

## Run

```sh
cargo run
```

## Keys

- `j` or Down: select next application.
- `k` or Up: select previous application.
- `e`: open the selected ChezMoi source path in `$EDITOR`; uses `nvim` when
  `$EDITOR` is unset.
- `a`: apply the selected target through ChezMoi.
- `d`: scan top-level `~/.config` entries with `fff-search`; add only new,
  ChezMoi-managed applications to LazyConfig's ChezMoi source registry.
- `r`: refresh registry state from ChezMoi.
- `q`: quit.
