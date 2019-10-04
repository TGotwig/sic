use peekmore::PeekMoreIterator;
use crate::cli_parse::{ParserError, Core};

pub mod id {
    pub const PRESERVE_ASPECT_RATIO: &str = "preserve-aspect-ratio";
    pub const SAMPLING_FILTER: &str = "sampling-filter";
}

use crate::cli_parse::parse_to::ParsePerTypeError;
use crate::cli_parse::core::op::Op;

// A modifier consists of the following syntax:
// `set <FOR_OP> <MODIFIER> <ARG...>`
#[derive(Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Modifier {
    PreserveAspectRatio,
    SamplingFilter(String),
}

impl Modifier {
    pub fn modifier_start(input: &str) -> bool {
        input.starts_with("set")
    }

    pub fn modifier_for<I: Iterator>(iter: &mut PeekMoreIterator<I>) -> Option<Result<Core, ParserError>>
    where I::Item: AsRef<str> {
        let core = iter.next()
            .ok_or(ParserError::PPTE(ParsePerTypeError::InvalidModifierSyntax))
            .and_then(|v| {
                let op = v.as_ref();

                if Op::is_some(op) {
                    let modifier = Modifier::consume_args(iter)?;
                    Ok(Core::SetModifier(0, modifier))
                } else {
                    Err(ParserError::PPTE(ParsePerTypeError::InvalidOperationForModifier))
                }
            });

        Some(core)
    }


    fn consume_args<I: Iterator>(
        iter: &mut PeekMoreIterator<I>
    ) -> Result<Self, ParserError>
        where
            I::Item: AsRef<str>,
    {
        let which = iter.next().ok_or(ParserError::PPTE(ParsePerTypeError::InvalidModifierSyntax))?;

        match which.as_ref() {
            id::PRESERVE_ASPECT_RATIO => Ok(Self::PreserveAspectRatio),
            id::SAMPLING_FILTER => {
                let filter = iter.next()
                    .ok_or(ParserError::PPTE(ParsePerTypeError::ModifierArgumentExpected))?;
                Ok(Self::SamplingFilter(filter.as_ref().to_string()))
            },
            _ => Err(ParserError::PPTE(ParsePerTypeError::InvalidModifierSyntax)),
        }
    }
}