use crate::cli_parse::{ParserError, Modifier};
use crate::cli_parse::parse_to::ParseFromIter;
use crate::cli_parse::core::numbers::{F32, I32, U32};
use std::collections::HashMap;
use peekmore::PeekMoreIterator;

#[derive(Debug, EnumString, AsRefStr, EnumVariantNames, ToString)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[strum(serialize_all = "kebab-case")]
pub enum Op {
    Blur(F32),
    Brighten(I32),
    Contrast(F32),
    Crop((U32, U32, U32, U32)),
    Filter3x3([F32; 9]),
    FlipHorizontal,
    FlipVertical,
    Grayscale,
    HueRotate(I32),
    Invert,
    Resize((U32, U32)),
    Rotate90,
    Rotate180,
    Rotate270,
    Unsharpen((F32, I32)),
}

impl Op {
    pub fn is_some(input: &str) -> bool {
        Op::variants().contains(&input)
    }

    pub fn from_str<I: Iterator>(
        iter: &mut PeekMoreIterator<I>,
        input: &str,
    ) -> Option<Result<Self, ParserError>>
        where
            I::Item: AsRef<str> + std::fmt::Debug,
    {
        match input {
            "blur" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: F32| Op::Blur(arg));

                Some(result)
            }
            "brighten" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: i32| Op::Brighten(arg));

                Some(result)
            }
            "contrast" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: F32| Op::Contrast(arg));

                Some(result)
            }
            "crop" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: (u32, u32, u32, u32)| Op::Crop(arg));

                Some(result)
            }
            "filter3x3" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: [F32; 9]| Op::Filter3x3(arg));

                Some(result)
            }
            "flip-horizontal" => Some(Ok(Op::FlipHorizontal)),
            "flip-vertical" => Some(Ok(Op::FlipVertical)),
            "grayscale" => Some(Ok(Op::Grayscale)),
            "hue-rotate" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: i32| Op::HueRotate(arg));

                Some(result)
            }
            "invert" => Some(Ok(Op::Invert)),
            "resize" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: (u32, u32)| Op::Resize(arg));

                Some(result)
            }
            "rotate90" => Some(Ok(Op::Rotate90)),
            "rotate180" => Some(Ok(Op::Rotate180)),
            "rotate270" => Some(Ok(Op::Rotate270)),
            "unsharpen" => {
                let result: Result<Self, ParserError> = ParseFromIter::parse(iter)
                    .map_err(|err| ParserError::PPTE(err))
                    .map(|arg: (F32, i32)| Op::Unsharpen(arg));

                Some(result)
            }
            _ => None,
        }
    }
}
