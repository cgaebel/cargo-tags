cargo-tags [![Build Status](https://travis-ci.org/cgaebel/cargo-tags.svg?branch=master)](https://travis-ci.org/cgaebel/cargo-tags)
=====================

A tool to graph transitive dependencies for Rust projects using Cargo

Installation
------------
Installation should be familiar to Cargo users. In this project's
directory, build the project and then add the binary to your `PATH`.

```sh
git clone git@github.com:cgaebel/cargo-tags.git
cargo build --release
export PATH=$PATH:$(pwd)/target/release
```

Usage
-----
In a Rust project using Cargo, run the following commands (assuming
cargo-tags and exuberant ctags are on your PATH)

```sh
cargo build # If you don't have a Cargo.lock file
cargo tags # cargo tags -e for emacs tags.
```

A tags will will be generated in your working directory.
