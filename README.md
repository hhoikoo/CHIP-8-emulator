# CHIP-8 Emulator

A CHIP-8 emulator in Rust.

Based on the CHIP-8 CPU emulator example in chapter 5 of *Rust in Action* by Tim McNamara.

## Status

Currently supported instructions:

| Opcode | Meaning |
| --- | --- |
| `00EE` | return |
| `1NNN` | jump to `NNN` |
| `2NNN` | call `NNN` |
| `3XNN` | skip next if `VX == NN` |
| `4XNN` | skip next if `VX != NN` |
| `5XY0` | skip next if `VX == VY` |
| `6XNN` | `VX = NN` |
| `7XNN` | `VX += NN` |
| `8XY4` | `VX += VY`, carry in `VF` |
| `9XY0` | skip next if `VX != VY` |
| `BNNN` | jump to `NNN + V0` (quirk: `XNN + VX`) |

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
