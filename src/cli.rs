use std::{error::Error, ffi::OsString, fmt, num::NonZeroUsize, path::PathBuf};

use chip8_emulator::cpu::{DisplayWaitQuirk, Quirks};

pub const DEFAULT_INSTRUCTIONS_PER_FRAME: NonZeroUsize =
    NonZeroUsize::new(12).expect("12 is not 0");
pub const USAGE: &str =
    "usage: chip8-emulator [<rom> [--speed <instructions per frame>] [--draw-immediately]]";

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub rom: Option<PathBuf>,
    pub instructions_per_frame: NonZeroUsize,
    pub quirks: Quirks,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UsageError {
    MissingSpeed,
    BadSpeed { value: OsString },
    UnknownFlag { flag: String },
    ExtraArgument { argument: OsString },
}

impl Options {
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, UsageError> {
        let mut options = Options {
            rom: None,
            instructions_per_frame: DEFAULT_INSTRUCTIONS_PER_FRAME,
            quirks: Quirks::default(),
        };
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--speed") => {
                    let value = args.next().ok_or(UsageError::MissingSpeed)?;
                    options.instructions_per_frame = parse_speed(value)?;
                }
                Some("--draw-immediately") => {
                    options.quirks.display_wait = DisplayWaitQuirk::DrawImmediately;
                }
                Some(flag) if flag.starts_with("--") => {
                    return Err(UsageError::UnknownFlag {
                        flag: flag.to_string(),
                    });
                }
                _ if options.rom.is_none() => options.rom = Some(PathBuf::from(arg)),
                _ => return Err(UsageError::ExtraArgument { argument: arg }),
            }
        }
        Ok(options)
    }
}

fn parse_speed(value: OsString) -> Result<NonZeroUsize, UsageError> {
    value
        .to_str()
        .and_then(|text| text.parse().ok())
        .ok_or(UsageError::BadSpeed { value })
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UsageError::MissingSpeed => {
                write!(f, "--speed needs a number of instructions per frame")
            }
            UsageError::BadSpeed { value } => write!(
                f,
                "--speed {} is not a whole number of instructions per frame above 0",
                value.display()
            ),
            UsageError::UnknownFlag { flag } => write!(f, "unknown option {flag}"),
            UsageError::ExtraArgument { argument } => {
                write!(
                    f,
                    "only one ROM can run, but {} was also given",
                    argument.display()
                )
            }
        }?;
        write!(f, "\n{USAGE}")
    }
}

impl Error for UsageError {}

#[cfg(test)]
mod tests;
