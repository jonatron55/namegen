Random Name Generator
=====================

This project generates random names using a variety of methods, including Markov
chains, for use in games, stories, and so on.

A live version of the Web UI can be found at <https://namegen.jonatron.ca>.

Configuration
-------------

Generators are created using [XML configuration] files, which can combine multiple components to create complex generators. Examples are provided in the [`configs/`]
directory and the [`docs/`] directory provides a detailed explanation of the
format.

Prerequisites
-------------

1. Install Git LFS from <https://git-lfs.github.com/>. The repository will not
   clone correctly without it.
2. Install `rustup` from <https://www.rust-lang.org/tools/install>.
3. Install the default Rust toolchain using `rustup install stable`.
4. Add the `wasm32-unknown-unknown` target to your Rust toolchain using
   `rustup target add wasm32-unknown-unknown`.
5. Install `trunk` using `cargo install trunk`.

Command-line interface
----------------------

To build the command-line interface, run `cargo build --release --bin namegen`.
The resulting binary will be located at `target/release/namegen`. You can run it
directly from there, or use `cargo run --release --bin namegen` to build and run
it in one step.

Prebuilt binaries for Windows, Linux, and macOS are available in the [releases]
section of this repository.

Web interface
-------------

To run the web interface, navigate to the `namegen-webui/` directory and run
`trunk serve --open`. This will start a local web server and open the web
interface in your default browser.

To build the web interface for production, run `trunk build --release`. The
resulting files will be located in the `dist` directory and can be served by any
static file server.

See also
--------

- [XML configuration] for details on the XML configuration format, which can be
  used to create complex generators with multiple components.

[`configs/`]: /configs/
[`docs/`]: /docs/
[releases]: https://github.com/jonatron55/namegen/releases/latest
[XML configuration]: /docs/config.md
