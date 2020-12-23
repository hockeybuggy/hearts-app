
I ran this locally with:

```
cargo watch -i .gitignore -i "pkg/*" -s "wasm-pack build --target web --out-name wasm --out-dir ./static"
```

and in another tmux pane:

```
cd static
python3 -m http
```

This required:

```
cargo install cargo-watch
cargo install cargo-wasm-pack
```
