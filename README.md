# minesweeper

## Web page
Try it out [here](https://saecki.github.io/minesweeper).

### Build
1. Install [trunk](https://trunkrs.dev/)
2. `cd` into the `web` directory
3. run `trunk serve --release` to build and serve the web page
4. open <https://localhost:8080>

## Desktop app

## Build
1. Install [rust](https://www.rust-lang.org/tools/install)
2. (On linux) Install development headers
    - Ubuntu: `sudo apt install libxcb-shape0-dev libxcb-xfixes0-dev libssl-dev libgtk-3-dev`
    - Fedora: `sudo dnf install pkg-config openssl-devel gtk3-devel`
3. Compile and run: `cargo run --release`
