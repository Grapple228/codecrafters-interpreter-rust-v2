# About

Second attempt to solve the challenge
With fixed errors, better structure, error handling, tests for modules

Done:
-- Variables of different types (Boolean, Number, String, Nil)
-- Builtin functions
-- User defined functions
-- While and for loops
-- If-else branching

This challenge follows the book

[Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom.

In this challenge you'll build an interpreter for
[Lox](https://craftinginterpreters.com/the-lox-language.html), a simple
scripting language. Along the way, you'll learn about tokenization, ASTs,
tree-walk interpreters and more.

Before starting this challenge, make sure you've read the "Welcome" part of the
book that contains these chapters:

- [Introduction](https://craftinginterpreters.com/introduction.html) (chapter 1)
- [A Map of the Territory](https://craftinginterpreters.com/a-map-of-the-territory.html)
  (chapter 2)
- [The Lox Language](https://craftinginterpreters.com/the-lox-language.html)
  (chapter 3)

These chapters don't involve writing code, so they won't be covered in this
challenge. This challenge will start from chapter 4,
[Scanning](https://craftinginterpreters.com/scanning.html).

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

## Dev setup

Firstly install `cargo-watch`

```sh
cargo install cargo-watch
```

### For execution app on save, use command

```sh
cargo watch -q -c -w src/ -w .cargo/ -x run
```

### For execution test app on save, use command

```sh
cargo watch -q -c -w examples/ -w .cargo/ -x "run --example quick-dev"
```

### For execution tests on save, use command

```sh
cargo watch -q -c -x "test -q -- --nocapture"
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
