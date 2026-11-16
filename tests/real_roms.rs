use std::{fs, path::PathBuf};

use chip8_emulator::{
    cpu::{Cpu, DisplayWaitQuirk, Quirks},
    rom::Rom,
};

const INSTRUCTIONS_PER_FRAME: usize = 12;
const CHOSEN_PLATFORM_ADDR: usize = 0x1FF;
const CHIP8_PLATFORM: u8 = 1;

#[derive(Clone, Copy)]
enum Budget {
    Cycles(usize),
    Frames(usize),
}

impl Budget {
    fn scaled(self, f: impl Fn(usize) -> usize) -> Budget {
        match self {
            Budget::Cycles(n) => Budget::Cycles(f(n)),
            Budget::Frames(n) => Budget::Frames(f(n)),
        }
    }
}

struct TestRom {
    dir: &'static str,
    name: &'static str,
    budget: Budget,
    preset: Option<(usize, u8)>,
}

const TEST_ROMS: [TestRom; 5] = [
    TestRom {
        dir: "timendus",
        name: "1-chip8-logo",
        budget: Budget::Cycles(39),
        preset: None,
    },
    TestRom {
        dir: "timendus",
        name: "2-ibm-logo",
        budget: Budget::Cycles(20),
        preset: None,
    },
    TestRom {
        dir: "timendus",
        name: "3-corax+",
        budget: Budget::Cycles(306),
        preset: None,
    },
    TestRom {
        dir: "timendus",
        name: "4-flags",
        budget: Budget::Cycles(952),
        preset: None,
    },
    TestRom {
        dir: "timendus",
        name: "5-quirks",
        budget: Budget::Frames(300),
        preset: Some((CHOSEN_PLATFORM_ADDR, CHIP8_PLATFORM)),
    },
];

fn rom_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/roms")
}

fn golden_path(rom: &TestRom) -> PathBuf {
    rom_dir().join("golden").join(format!("{}.txt", rom.name))
}

fn load(rom: &TestRom) -> Rom {
    let path = rom_dir().join(rom.dir).join(format!("{}.ch8", rom.name));
    let loaded = Rom::from_file(&path).unwrap();
    match rom.preset {
        Some((addr, byte)) => loaded.with_byte(addr, byte).unwrap(),
        None => loaded,
    }
}

fn screen_after(rom: &TestRom, budget: Budget) -> String {
    let fail = |err| panic!("{}: {err}", rom.name);
    let cpu = match budget {
        Budget::Cycles(cycles) => {
            let quirks = Quirks {
                display_wait: DisplayWaitQuirk::DrawImmediately,
                ..Quirks::default()
            };
            let mut cpu = Cpu::with_quirks(&load(rom), quirks);
            cpu.run_for(cycles).unwrap_or_else(fail);
            cpu
        }
        Budget::Frames(frames) => {
            let mut cpu = Cpu::new(&load(rom));
            for _ in 0..frames {
                cpu.run_for(INSTRUCTIONS_PER_FRAME).unwrap_or_else(fail);
                cpu.tick_timers();
            }
            cpu
        }
    };
    cpu.framebuffer().to_string()
}

#[test]
fn roms_draw_their_golden_image_within_their_budget() {
    for rom in &TEST_ROMS {
        let got = screen_after(rom, rom.budget);
        let want = fs::read_to_string(golden_path(rom)).unwrap();
        assert!(got == want, "{} drew:\n{got}\nexpected:\n{want}", rom.name);
    }
}

#[test]
fn roms_need_their_whole_budget() {
    for rom in &TEST_ROMS {
        assert_ne!(
            screen_after(rom, rom.budget.scaled(|n| n - 1)),
            screen_after(rom, rom.budget),
            "{} finishes its image early",
            rom.name,
        );
    }
}

#[test]
fn roms_keep_their_image_after_their_budget() {
    for rom in &TEST_ROMS {
        assert_eq!(
            screen_after(rom, rom.budget),
            screen_after(rom, rom.budget.scaled(|n| n * 10)),
            "{} still changes the screen after its budget",
            rom.name,
        );
    }
}

#[test]
#[ignore = "rewrites tests/roms/golden; check the new images by eye"]
fn regenerate_golden_images() {
    for rom in &TEST_ROMS {
        fs::write(golden_path(rom), screen_after(rom, rom.budget)).unwrap();
    }
}
