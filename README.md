# CHIP-8 Emulator

A CHIP-8 emulator in Rust.

Based on the CHIP-8 CPU emulator example in chapter 5 of *Rust in Action* by Tim McNamara.

## Status

Currently supported instructions:

| Opcode | Meaning |
| --- | --- |
| `00E0` | clear the screen |
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
| `CXNN` | `VX` = random byte `& NN` |
| `DXYN` | XOR-draw `N` sprite rows from `I` at `(VX, VY)`, `VF = 1` if a pixel turned off; clipped at the edges (quirk: wrap); then waits for the next 60 Hz frame (quirk: draw immediately) |
| `EX9E` | skip next if key `VX` is down (`VX > 0xF` is an error) |
| `EXA1` | skip next if key `VX` is up (`VX > 0xF` is an error) |
| `FX07` | `VX` = delay timer |
| `FX0A` | wait for a key to be pressed and released, then `VX` = that key |
| `FX15` | delay timer = `VX` |
| `FX18` | sound timer = `VX` |
| `FX1E` | `I += VX` |
| `FX29` | `I` = address of the font sprite for digit `VX` (`VX > 0xF` is an error) |
| `FX33` | store `VX` as 3 decimal digits at `I`, `I+1`, `I+2` |
| `FX55` | store `V0`–`VX` at `I`, then `I += X + 1` (quirk: `I` unchanged) |
| `FX65` | load `V0`–`VX` from `I`, then `I += X + 1` (quirk: `I` unchanged) |

The rest, and the planned assembler, are in [TODO.md](TODO.md).

## Run

### Emulator

```sh
cargo run -- tests/roms/timendus/2-ibm-logo.ch8
```

This opens a window that shows the screen at ×16 and runs 12 instructions per 60 Hz frame. Close it with Escape, the close button or Ctrl-C. It prints the registers when it stops, plus the error if the ROM failed.

`cargo run` with no ROM runs a small built-in demo without a window and prints `5 + (2 * 10) + (2 * 10) = 45`.

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

[MIT](LICENSE). The test ROMs in `tests/roms/timendus/` are GPL-3.0 (see [`tests/roms/README.md`](tests/roms/README.md)).
