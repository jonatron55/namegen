Random Name Generator
=====================

This project generates random names using a variety of methods, including Markov
chains, for use in games, stories, and so on.

Getting started
---------------

1. Install the Rust toolchain from <https://www.rust-lang.org/tools/install>.
2. Clone this repository and navigate to the project directory.
3. Build the project using `cargo build` or `cargo build --release`. The first
   build may take some time as dependencies are downloaded and compiled. The
   output will be placed in the `target/debug` or `target/release` directory,
   accordingly.
4. To run the generator, use `cargo run -- [args]`. See the [Usage](#usage)
   section below for details on the available arguments. Notice that the `--` is
   required to separate the arguments for the generator from the arguments for
   the `cargo` command.

See also
--------

- [XML configuration] for details on the XML configuration format, which can be
  used to create complex generators with multiple components.

[XML configuration]: docs/config.md
[`<Markov>`]: docs/config.md#markov
