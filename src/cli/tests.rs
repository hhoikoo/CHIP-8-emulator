use super::*;

fn parse(args: &[&str]) -> Result<Options, UsageError> {
    Options::parse(args.iter().map(OsString::from))
}

fn speed(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).unwrap()
}

#[test]
fn no_arguments_mean_no_rom_and_the_defaults() {
    assert_eq!(
        parse(&[]),
        Ok(Options {
            rom: None,
            instructions_per_frame: speed(12),
            quirks: Quirks::default(),
        })
    );
}

#[test]
fn the_first_plain_argument_is_the_rom() {
    let options = parse(&["pong.ch8"]).unwrap();
    assert_eq!(options.rom, Some(PathBuf::from("pong.ch8")));
    assert_eq!(
        options.instructions_per_frame,
        DEFAULT_INSTRUCTIONS_PER_FRAME
    );
    assert_eq!(options.quirks, Quirks::default());
}

#[test]
fn flags_can_come_before_or_after_the_rom() {
    let expected = Options {
        rom: Some(PathBuf::from("pong.ch8")),
        instructions_per_frame: speed(30),
        quirks: Quirks {
            display_wait: DisplayWaitQuirk::DrawImmediately,
            ..Quirks::default()
        },
    };
    assert_eq!(
        parse(&["pong.ch8", "--speed", "30", "--draw-immediately"]),
        Ok(expected)
    );
    assert_eq!(
        parse(&["--draw-immediately", "--speed", "30", "pong.ch8"]).unwrap(),
        parse(&["pong.ch8", "--speed", "30", "--draw-immediately"]).unwrap()
    );
}

#[test]
fn the_last_speed_wins() {
    let options = parse(&["--speed", "5", "--speed", "7"]).unwrap();
    assert_eq!(options.instructions_per_frame, speed(7));
}

#[test]
fn speed_needs_a_positive_whole_number() {
    assert_eq!(parse(&["--speed"]), Err(UsageError::MissingSpeed));
    for bad in ["0", "-1", "fast", "1.5", ""] {
        assert_eq!(
            parse(&["--speed", bad]),
            Err(UsageError::BadSpeed {
                value: OsString::from(bad)
            })
        );
    }
}

#[test]
fn unknown_flags_and_a_second_rom_are_errors() {
    assert_eq!(
        parse(&["--turbo"]),
        Err(UsageError::UnknownFlag {
            flag: "--turbo".to_string()
        })
    );
    assert_eq!(
        parse(&["a.ch8", "b.ch8"]),
        Err(UsageError::ExtraArgument {
            argument: OsString::from("b.ch8")
        })
    );
}

#[test]
fn errors_end_with_the_usage_line() {
    assert_eq!(
        UsageError::BadSpeed {
            value: OsString::from("0")
        }
        .to_string(),
        format!("--speed 0 is not a whole number of instructions per frame above 0\n{USAGE}")
    );
    assert_eq!(
        UsageError::MissingSpeed.to_string(),
        format!("--speed needs a number of instructions per frame\n{USAGE}")
    );
}
