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

## Real ROMs (not used yet)

To be added as tests once every instruction is implemented. None are in the repo yet. Before adding one, pin the raw URL to a commit and record a checksum.

This repo is MIT, so only ROMs with a compatible license can be committed:

| ROM | Source | License | Can commit? | Needs | How to check it |
|---|---|---|---|---|---|
| `test_opcode.ch8` (corax89) | [corax89/chip8-test-rom](https://github.com/corax89/chip8-test-rom) | MIT | yes, with its notice | skips, `8XY_`, `ANNN`, `FX33`, `FX55`, `DXYN` | golden framebuffer |
| chip8Archive games, e.g. `br8kout` | [JohnEarnest/chip8Archive](https://github.com/JohnEarnest/chip8Archive) | CC0 (stated in the README) | yes | `CXNN`, keypad, timers | smoke test only: needs a seeded RNG and scripted keys |
| `1-chip8-logo.ch8` (Timendus) | [Timendus/chip8-test-suite](https://github.com/Timendus/chip8-test-suite) | GPL-3.0 | decide first | `00E0`, `6XNN`, `ANNN`, `DXYN`, `1NNN` | framebuffer after 39 cycles |
| `2-ibm-logo.ch8` (Timendus) | same | GPL-3.0; the original IBM logo bytes have no known license | decide first | adds `7XNN` and unaligned `DXYN` | framebuffer after 20 cycles |
| `3-corax+.ch8` (Timendus) | same | GPL-3.0 | decide first | skips, calls, `8XY_`, `FX1E`, `FX33`, `FX55`, `FX65` | golden framebuffer of the pass/fail marks |
| `4-flags.ch8` (Timendus) | same | GPL-3.0 | decide first | `8XY_` with `VF` as input and output | golden framebuffer |
| `5-quirks.ch8` (Timendus) | same | GPL-3.0 | decide first | timers, `BNNN`, `FX55`/`FX65`, display wait | write `1` to `0x1FF` first to pick CHIP-8; golden framebuffer |
| `6-keypad.ch8`, `7-beep.ch8` (Timendus) | same | GPL-3.0 | decide first | keypad, sound timer | needs scripted keys or timer checks |
| Pong, Maze and other classics | [kripod/chip8-roms](https://github.com/kripod/chip8-roms) | unclear | no | varies | local smoke tests only |

The order they'd become usable as opcodes land: `1-chip8-logo`, `2-ibm-logo`, `3-corax+` (with `test_opcode`), `4-flags`, `5-quirks`.

The GPL-3.0 ROMs could be downloaded by the test instead of committed, or kept in an untracked folder. Decide before adding them.
