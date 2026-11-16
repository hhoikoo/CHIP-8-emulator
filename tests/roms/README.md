# Test ROMs

## Generated ROMs (`generated/`)

Tiny `.ch8` files built only from the opcodes the emulator supports. Their source of truth is the listing in `tests/generated_roms.rs`:

- `rom_files_match_their_listings` fails if a file drifts from its listing.
- `cargo test --test generated_roms -- --ignored` rewrites the files from the listings.
- Each ROM has a cycle budget. A ROM that loops reaches its loop (a jump to itself) in exactly that many cycles and is checked by its registers. A ROM that fails fails on exactly its last cycle, and one cycle fewer runs without an error.

| ROM | Cycles | Outcome |
|---|---|---|
| `jump` | 2 | `V0 = 2`, after jumping over `V0 = 1` |
| `skips` | 4 | `V1 = 0` (skipped), `V2 = 1` (not skipped) |
| `zero_padding_is_an_unknown_opcode` | 2 | `V0 = 1`, then fails: zeroed memory decodes as unknown opcode `0x0000` |
| `load_and_add` | 2 | `V0 = 8` |
| `add_with_carry` | 3 | `V0 = 0x10`, `VF = 1` |
| `call_and_return` | 10 | `V0 = 45` (the demo program in one file) |
| `nested_calls` | 48 | `V0 = 16`, after 16 nested calls (the full stack) |
| `stack_overflow` | 17 | fails: stack overflow |
| `stack_underflow` | 1 | fails: stack underflow |
| `unknown_opcode` | 2 | fails: unknown opcode `0xF000` |

## Timendus test ROMs (`timendus/`)

Four ROMs from [Timendus/chip8-test-suite](https://github.com/Timendus/chip8-test-suite), copied unchanged from `bin/` at commit [`742e9ea`](https://github.com/Timendus/chip8-test-suite/tree/742e9eac9f5d44466c1c9d83fcaa8158c6683cb1/bin).

They are GPL-3.0, and their license is in `timendus/LICENSE`. They are test data only: nothing in the emulator links or embeds them, and the rest of this repo stays MIT.

`tests/timendus_roms.rs` runs each ROM for its cycle budget and compares the screen with `golden/<rom>.txt`, which has 32 lines of 64 `#` (lit) and `.` (dark) characters:

- The image appears on exactly the last cycle of the budget, and it doesn't change in 10× as many cycles.
- `cargo test --test timendus_roms -- --ignored` rewrites the golden images. Check them by eye afterwards.

| ROM | SHA-256 | Cycles | Image |
|---|---|---|---|
| `1-chip8-logo.ch8` | `15f7fb887ea4cb8e40615bb20b0cfd993ef55ca84c535c8c8f3fafa8bf129724` | 39 | the CHIP-8 logo (the budget matches the suite's docs) |
| `2-ibm-logo.ch8` | `00072a250d2f7ccaa3ecc0182bac73e63c168abab96d7ea2df5eba6a4da49067` | 20 | the IBM logo (the budget matches the suite's docs) |
| `3-corax+.ch8` | `1c7e14eae14d6d5e1e47693804110354cbc4081defe4e6e5d9167c25ffc7b4b0` | 306 | 22 pass marks and no fail marks |
| `4-flags.ch8` | `f00ddadd37bc878473de0c8f16faecf9985dea39036a3a796d551bc9fec47cfa` | 952 | 47 pass marks and no fail marks |

A pass mark is `#.#`, `##.`, `#..`, and a fail mark is an X (`#.#`, `.#.`, `#.#`). To check that a fail would show: with the `8XY4` carry broken on purpose, `4-flags` draws 2 fail marks.

## Other real ROMs (not used yet)

Before adding one, pin the raw URL to a commit and record a checksum.

This repo is MIT, so only ROMs with a compatible license can be committed:

| ROM | Source | License | Can commit? | Needs | How to check it |
|---|---|---|---|---|---|
| `test_opcode.ch8` (corax89) | [corax89/chip8-test-rom](https://github.com/corax89/chip8-test-rom) | MIT | yes, with its notice | skips, `8XY_`, `ANNN`, `FX33`, `FX55`, `DXYN` | golden framebuffer |
| chip8Archive games, e.g. `br8kout` | [JohnEarnest/chip8Archive](https://github.com/JohnEarnest/chip8Archive) | CC0 (stated in the README) | yes | `CXNN`, keypad, timers | smoke test only: needs a seeded RNG and scripted keys |
| `1-chip8-logo.ch8` (Timendus) | [Timendus/chip8-test-suite](https://github.com/Timendus/chip8-test-suite) | GPL-3.0 | **added** | `00E0`, `6XNN`, `ANNN`, `DXYN`, `1NNN` | framebuffer after 39 cycles |
| `2-ibm-logo.ch8` (Timendus) | same | GPL-3.0; the original IBM logo bytes have no known license | **added** | adds `7XNN` and unaligned `DXYN` | framebuffer after 20 cycles |
| `3-corax+.ch8` (Timendus) | same | GPL-3.0 | **added** | skips, calls, `8XY_`, `FX1E`, `FX33`, `FX55`, `FX65` | golden framebuffer of the pass/fail marks |
| `4-flags.ch8` (Timendus) | same | GPL-3.0 | **added** | `8XY_` with `VF` as input and output | golden framebuffer |
| `5-quirks.ch8` (Timendus) | same | GPL-3.0 | decide first | timers, `BNNN`, `FX55`/`FX65`, display wait | write `1` to `0x1FF` first to pick CHIP-8; golden framebuffer |
| `6-keypad.ch8`, `7-beep.ch8` (Timendus) | same | GPL-3.0 | decide first | keypad, sound timer | needs scripted keys or timer checks |
| Pong, Maze and other classics | [kripod/chip8-roms](https://github.com/kripod/chip8-roms) | unclear | no | varies | local smoke tests only |

The order they'd become usable as opcodes land: `1-chip8-logo`, `2-ibm-logo`, `3-corax+` (with `test_opcode`), `4-flags`, `5-quirks`.

The other GPL-3.0 Timendus ROMs go in `timendus/` the same way, under the same license.
