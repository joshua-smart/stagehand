use std::error::Error;
use std::path::PathBuf;

use nom::bytes::complete::take_while;
use nom::character::complete::alpha1;
use nom::character::complete::u8;
use nom::character::complete::u16;
use nom::character::complete::usize;
use nom::combinator::all_consuming;
use nom::combinator::eof;
use nom::combinator::opt;
use nom::combinator::value;
use nom::error::ErrorKind;
use nom::error::FromExternalError;
use nom::error::ParseError;
use nom::error::make_error;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::{IResult, Parser, branch::alt, bytes::complete::tag, character::complete::space1};

use crate::data_structures::address::Address;
use crate::data_structures::address::AddressRange;
use crate::data_structures::address::AddressSet;
use crate::data_structures::index::Index;
use crate::data_structures::level::Level;
use crate::data_structures::level::LevelRange;
use crate::data_structures::level::LevelSet;
use crate::data_structures::universe::Universe;

#[derive(Debug)]
pub enum Command {
    Quit,
    Save {
        path: Option<PathBuf>,
    },
    SetAddress {
        address_set: AddressSet,
        level_set: LevelSet,
    },
    ClearAddress {
        address_set: AddressSet,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ParseCommandError<I> {
    #[error("{0}")]
    Nom(nom::error::Error<I>),
    #[error("invalid universe: {0}")]
    InvalidUniverse(u16),
    #[error("invalid index: {0}")]
    InvalidIndex(u16),
    #[error("invalid address range: {0}-{1} (step: {2:?})")]
    InvalidAddressRange(Index, Index, Option<usize>),
    #[error("invalid level range: {0}-{1}")]
    InvalidLevelRange(Level, Level),
    #[error("{2}")]
    External(I, ErrorKind, String),
}

impl<I> ParseError<I> for ParseCommandError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self::Nom(make_error(input, kind))
    }

    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> From<ParseCommandError<I>> for nom::Err<ParseCommandError<I>> {
    fn from(value: ParseCommandError<I>) -> Self {
        nom::Err::Error(value)
    }
}

impl<I, E: Error> FromExternalError<I, E> for ParseCommandError<I> {
    fn from_external_error(input: I, kind: ErrorKind, e: E) -> Self {
        Self::External(input, kind, e.to_string())
    }
}

type Result<I, O> = IResult<I, O, ParseCommandError<I>>;

pub fn parse_args(i: &str) -> Result<&str, (&str, Vec<&str>)> {
    terminated(
        (
            alpha1,
            many0(preceded(
                space1,
                alt((
                    delimited(tag("\""), take_while(|c: char| c != '"'), tag("\"")),
                    take_while(|c: char| !c.is_whitespace()),
                )),
            )),
        ),
        eof,
    )
    .parse(i)
}

pub fn parse_command(i: &str) -> Result<&str, Command> {
    let (_, (command, args)) = parse_args.parse_complete(i)?;

    Ok((
        "",
        match command {
            "set" => {
                let (_, address_set) = all_consuming(parse_address_set).parse_complete(args[0])?;
                let (_, level_set) = all_consuming(parse_level_set).parse_complete(args[1])?;
                Command::SetAddress {
                    address_set,
                    level_set,
                }
            }
            "clear" => {
                let (_, address_set) = args
                    .first()
                    .map(|s| all_consuming(parse_address_set).parse_complete(s))
                    .unwrap_or(Ok(("", AddressSet::all(Universe::ONE))))?;
                Command::ClearAddress { address_set }
            }
            "save" => Command::Save {
                path: args.first().map(PathBuf::from),
            },
            _ => todo!(),
        },
    ))

    // terminated(alt((parse_set, parse_clear, parse_save)), (space0, eof)).parse(i)
}

fn parse_address_set(i: &str) -> Result<&str, AddressSet> {
    (
        opt(terminated(parse_universe, tag("/"))),
        parse_index,
        opt((
            preceded(tag("-"), parse_index),
            opt(preceded(tag("|"), usize)),
        )),
    )
        .map_res(|(u_opt, start, end_opt)| {
            let universe = u_opt.unwrap_or(Universe::new(1).unwrap());
            match end_opt {
                None => Ok(AddressSet::Single(Address {
                    universe,
                    index: start,
                })),
                Some((end, step_opt)) => (match step_opt {
                    Some(step) => AddressRange::with_step(universe, start, end, step),
                    None => AddressRange::new(universe, start, end),
                })
                .ok_or(ParseCommandError::<&str>::InvalidAddressRange(
                    start, end, step_opt,
                ))
                .map(AddressSet::Range),
            }
        })
        .parse(i)
}

fn parse_universe(i: &str) -> Result<&str, Universe> {
    u16.map_res(|u| Universe::new(u).ok_or(ParseCommandError::<&str>::InvalidUniverse(u)))
        .parse(i)
}

fn parse_index(i: &str) -> Result<&str, Index> {
    u16.map_res(|i| Index::new(i).ok_or(ParseCommandError::<&str>::InvalidIndex(i)))
        .parse(i)
}

fn parse_level_set(i: &str) -> Result<&str, LevelSet> {
    let single = |i| {
        alt((
            value(Level::OUT, tag("o")),
            value(Level::FULL, tag("f")),
            terminated(u8, tag("%")).map(|p| {
                let p = (f64::from(p) * 255_f64 / 100_f64).round() as u8;
                Level::new(p)
            }),
            u8.map(Level::new),
        ))
        .parse(i)
    };

    (single, opt(preceded(tag("-"), single)))
        .map(|(start, opt_end)| match opt_end {
            None => LevelSet::Single(start),
            Some(end) => LevelSet::Range(LevelRange { start, end }),
        })
        .parse(i)
}
