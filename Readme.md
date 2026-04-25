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

Usage
-----

```plaintext
namegen.exe [options] [<file>]
```

### Arguments ###

- `<file>`: Path to generator configuration.

  This file may be either a plain text file or an [XML configuration]. The type
  will be inferred from the file extension if possible, or by the presence of an
  XML signature in the file contents.

  Plain text files will create a Markov generator using the default options
  described in [`<Markov>`] and will be trained on the whitespace-separated
  words in the file.

  XML files should follow the format described in [XML configuration] and can be
  used to create more complex generators with multiple components.

  Built-in configurations are also available. The following names can be used to
  reference them:

  - `default`: A configuration that generates amusing person names. Excellent
    if you're planning on running for MP under the Silly Party.
  - `thing`: A configuration that generates amusing names for objects or
    concepts. Perfect for naming components of your turboencabulator.

  If a path is not provided, the `default` builtin configuration will be used.
  To use a configuration file that has the same name as a builtin, prefix it
  with `./` or another path component.

### Options ###

- `-n`, `--count <N>`: Number of names to generate. Default is 1.

- `-s`, `--seed <N>`: Seed for the random number generator. Default is a random
  seed.

- `-a`, `--analyze`:  Analyze the given config file without generating names.

  This will output statistics about the Markov chain frequencies and
  combinatorics for the given config.

- `-h`, `--help`: Print help information.

- `-V`, `--version`: Print version information.

See also
--------

- [XML configuration] for details on the XML configuration format, which can be
  used to create complex generators with multiple components.

[XML configuration]: docs/config.md
[`<Markov>`]: docs/config.md#markov
