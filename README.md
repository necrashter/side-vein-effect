# Side Vein Effect

A video game developed during [Bevy Jam 3](https://itch.io/jam/bevy-jam-3).

You play as a nanomachine drug in a patient's bloodstream.
You need to shoot the germs and protect the blood cells.
If you miss the germs, it will create side-effects on the sides of the vein.

![Cover Image](assets/graphics/cover.png)

## [Play on itch.io](https://necrashter.itch.io/side-vein-effect)

[Click to play on itch.io](https://necrashter.itch.io/side-vein-effect).

## Clone the repository

This repository uses Git-LFS to store the game assets. Make sure that Git-LFS is installed on your system.
You can use the following commands to clone the repository and pull the files stored by Git-LFS:
```sh
git clone https://github.com/necrashter/side-vein-effect
cd side-vein-effect
git lfs install
git lfs pull
git fetch
```

## Run Localy

It can be built and run like a regular Bevy game:
```sh
# Run in debug mode:
cargo run

# Run in debug mode but compile using dynamic linking.
# This makes subsequent builds faster.
cargo run --features bevy/dynamic_linking

# Run in release mode with all optimizations.
cargo run --release
```

## Deploy to Web

Install the prerequisites:
```sh
rustup target install wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```

Build:

```sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-name game --out-dir web --target web target/wasm32-unknown-unknown/release/side-vein-effect.wasm
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


# License

The following resources are from third-parties:
- `assets/fonts/Kanit-Regular.ttf` is licensed under Open Font License.
- Files under `web/` are from [`bevy_github_ci_template`](https://github.com/bevyengine/bevy_github_ci_template), licensed under the same license as Bevy.

The remaining files are my own original work:
- All code (everything in `src/`, `Cargo.toml`) are under [MIT License](LICENSE).
- All assets (everything in `assets/graphics` and `assets/music`) are under [CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/).
