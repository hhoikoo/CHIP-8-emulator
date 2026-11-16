use std::{fs, path::PathBuf};

use chip8_emulator::{
    cpu::{Cpu, DisplayWaitQuirk, Quirks},
    rom::Rom,
};

struct TestRom {
    name: &'static str,
    cycles: usize,
}

const TEST_ROMS: [TestRom; 4] = [
    TestRom {
        name: "1-chip8-logo",
        cycles: 39,
    },
    TestRom {
        name: "2-ibm-logo",
        cycles: 20,
    },
    TestRom {
        name: "3-corax+",
        cycles: 306,
    },
    TestRom {
        name: "4-flags",
        cycles: 952,
    },
];

fn rom_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/roms")
}

fn golden_path(rom: &TestRom) -> PathBuf {
    rom_dir().join("golden").join(format!("{}.txt", rom.name))
}

fn screen_after(rom: &TestRom, cycles: usize) -> String {
    let path = rom_dir().join("timendus").join(format!("{}.ch8", rom.name));
    let quirks = Quirks {
        display_wait: DisplayWaitQuirk::DrawImmediately,
        ..Quirks::default()
    };
    let mut cpu = Cpu::with_quirks(&Rom::from_file(&path).unwrap(), quirks);
    cpu.run_for(cycles)
        .unwrap_or_else(|err| panic!("{}: {err}", rom.name));
    cpu.framebuffer().to_string()
}

#[test]
fn roms_draw_their_golden_image_within_their_cycle_budget() {
    for rom in &TEST_ROMS {
        let got = screen_after(rom, rom.cycles);
        let want = fs::read_to_string(golden_path(rom)).unwrap();
        assert!(got == want, "{} drew:\n{got}\nexpected:\n{want}", rom.name);
    }
}

#[test]
fn roms_need_their_whole_cycle_budget() {
    for rom in &TEST_ROMS {
        assert_ne!(
            screen_after(rom, rom.cycles - 1),
            screen_after(rom, rom.cycles),
            "{} finishes its image before cycle {}",
            rom.name,
            rom.cycles
        );
    }
}

#[test]
fn roms_keep_their_image_after_their_cycle_budget() {
    for rom in &TEST_ROMS {
        assert_eq!(
            screen_after(rom, rom.cycles),
            screen_after(rom, rom.cycles * 10),
            "{} still changes the screen after {} cycles",
            rom.name,
            rom.cycles
        );
    }
}

#[test]
#[ignore = "rewrites tests/roms/golden; check the new images by eye"]
fn regenerate_golden_images() {
    for rom in &TEST_ROMS {
        fs::write(golden_path(rom), screen_after(rom, rom.cycles)).unwrap();
    }
}
