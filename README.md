# CHIP-8 Emulator

A CHIP-8 emulator in Rust.

Based on the CHIP-8 CPU emulator example in chapter 5 of *Rust in Action* by Tim McNamara.

## Status

Currently supported instructions:

| Opcode | Meaning |
| --- | --- |
| `0000` | halt (our own stop instruction) |
| `00EE` | return |
| `2NNN` | call `NNN` |
| `6XNN` | `VX = NN` |
| `7XNN` | `VX += NN` |
| `8XY4` | `VX += VY`, carry in `VF` |

The rest, and the planned assembler, are in [TODO.md](TODO.md).

## Run

### Emulator

```sh
cargo run
```

### Assembler

TODO

## Develop

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The first `cargo test` installs a pre-commit hook (via [husky-rs](https://crates.io/crates/husky-rs)) that runs the checks above.

## License

[MIT](LICENSE)
