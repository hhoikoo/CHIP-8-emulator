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
| `8XY0` | `VX = VY` |
| `8XY1` | `VX \|= VY`, `VF = 0` (quirk: `VF` kept) |
| `8XY2` | `VX &= VY`, `VF = 0` (quirk: `VF` kept) |
| `8XY3` | `VX ^= VY`, `VF = 0` (quirk: `VF` kept) |
| `8XY4` | `VX += VY`, carry in `VF` |
| `8XY5` | `VX -= VY`, `VF = 1` if no borrow |
| `8XY6` | `VX = VY >> 1`, `VF` = bit shifted out (quirk: shift `VX` in place) |
| `8XY7` | `VX = VY - VX`, `VF = 1` if no borrow |
| `8XYE` | `VX = VY << 1`, `VF` = bit shifted out (quirk: shift `VX` in place) |
| `9XY0` | skip next if `VX != VY` |
| `ANNN` | `I = NNN` |
| `BNNN` | jump to `NNN + V0` (quirk: `XNN + VX`) |
| `FX1E` | `I += VX` |
| `FX33` | store `VX` as 3 decimal digits at `I`, `I+1`, `I+2` |
| `FX55` | store `V0`–`VX` at `I`, then `I += X + 1` (quirk: `I` unchanged) |
| `FX65` | load `V0`–`VX` from `I`, then `I += X + 1` (quirk: `I` unchanged) |

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
