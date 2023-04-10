# Side Vein Effect

Bevy Jam 3 entry.

You play as a nanomachine drug in a patient's bloodstream.
You need to shoot the germs and protect the blood cells.
If you miss the germs, it will create side-effects on the sides of the vein.

## Cloning and Running Localy

This repository uses `git-lfs`.

TODO

## Deploy to Web

Install the prerequisites:
```sh
rustup target install wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

Build:

```sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-name game --out-dir web --target web target/wasm32-unknown-unknown/release/side-effect.wasm
cp -r assets/ web/
```

Optionally, use wasm-opt to optimize the game. This decreases the size of the `.wasm` executable and also reduces the audio glitches caused by buffer underruns (which presumably happen because the web build is single-threaded).

```sh
# Install wasm-opt on Ubuntu
sudo apt install binaryen

cd web
wasm-opt -O -ol 100 -s 100 -o game_bg.wasm game_bg.wasm
```

You can use the HTTP server in Python to test the web build:
```sh
python3 -m http.server
```
Now navigate to http://0.0.0.0:8000/ with your web browser.

More resources for deploying to web:
- [release.yaml in `bevy_github_ci_template`](https://github.com/bevyengine/bevy_github_ci_template/blob/main/.github/workflows/release.yaml)
- [Unofficial Bevy Cheat Book](https://bevy-cheatbook.github.io/platforms/wasm.html)
