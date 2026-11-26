# gpui-shell

A spike: one window with a projects sidebar and a virtual list of made-up issue rows, on
gpui-component under Wayland, with the findings in `docs/findings/gpui.md`.

From this folder:

```sh
CARGO_TARGET_DIR="$PWD/target" nice -n 10 cargo build -j 4 --release
./target/release/gpui-shell
```
