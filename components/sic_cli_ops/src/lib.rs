#![deny(clippy::all)]

#[macro_use]
extern crate strum_macros;

#[cfg(test)]
#[macro_use]
extern crate parameterized;

use crate::errors::SicCliOpsError;
use crate::operations::OperationId;
use sic_image_engine::engine::Instr;
use std::fmt::Debug;
use strum::VariantNames;

pub mod errors;
pub mod named_value;
pub mod operations;
pub mod value_parser;

pub type TResult<T> = Result<T, SicCliOpsError>;

/// Parses cli image operation definitions to image engine image operations.
/// This parser however doesn't replace Clap, and specifically its validator.
/// For example, `--flip-horizontal 0` will not be allowed
/// by Clap's validator. The function below however does allow it, since we parse
/// only the amount of arguments we expect to receive, in this case 0.
/// Since we can rely on Clap, we left the added complexity out here.  
pub fn create_image_ops_cli<I: IntoIterator<Item = T>, T: AsRef<str> + Debug>(
    iter: I,
) -> TResult<Vec<Instr>> {
    let mut iter = iter.into_iter();
    let mut ast = empty_ast(iter.size_hint().1);

    while let Some(arg) = iter.next() {
        let arg = arg.as_ref();

        if arg.starts_with("--") && OperationId::VARIANTS.contains(&&arg[2..]) {
            let operation = OperationId::try_from_name(&arg[2..])?;
            let inputs = take_n(&mut iter, operation)?;
            ast.push(operation.create_instruction(inputs.iter().map(|v| v.as_ref()))?);
        }
        // else: skip
    }

    Ok(ast)
}

// unlike the function `create_image_ops_cli` above, this function doesn't expect components (consisting
// of image operation identifiers followed by their arguments). Instead it expects full operations,
// pre-split by ';' which script requires. An example of an iterator with components would be:
// `["blur", "1", "blur", "2"]` while an iterator with full operations will consist of:
// `["blur 1", "blur 2"]`. Here the original script would have been something like `blur 1; blur 2`
// TODO:
//  - This won't work without major hackiness, since named values accept spaces, and would be split
//    into separate components while they're not; same for paths which contains whitespace (in script;
//    not in cli version since the shell preprocesses our input there; we could do the same but then
//    why not bite the bullet and do a proper attempt instead of all these hacks to get a single
//    codebase for the parsing(-like) of image operations <3)
pub fn create_image_ops_script<I: IntoIterator<Item = T>, T: AsRef<str> + Debug>(
    iter: I,
) -> TResult<Vec<Instr>> {
    // HACK: avoid multi-pass iterator parsing and cherry picking iterator elements for unquoting:
    // This hack is used to so paths given as parameter through the script will be unquoted, and can
    // use the same parsing implementation for the cli ops as well as script. If we'll have a ton of
    // script specific code, we should change it to be a parameter for the parser or some other proper
    // solution, since this effectively sets a global read only value. When cli ops is used, a given  path,
    // e.g. `--diff "/my/path"` will be given as input without the quotation marks. When using script however,
    // for `diff "/my/path"` we would receive the string `"/my/path"` instead of `/my/path`. We could unquote each
    // path on our side, but with the current implementation, that would make this function rather messy, as we would
    // need to unquote every possible path (since we don't use a proper parser anymore, but use some string splitting
    // instead, we would have to do some chery picking on specific elements of our iterator where arguments are expected to
    // be paths, which also is non-ideal). Until we think of a better, proper fix, we'll use this lovely hack instead.
    std::env::set_var("SIC_SCRIPT_WORKAROUND_UNQUOTE", "1");

    let ops = iter
        .into_iter()
        .filter_map(|op| {
            let op: &str = op.as_ref();

            // ensure a script with empty statements would still be valid, e.g.
            // `blur 1;; blur 2` and `blur 1;` (ends with empty string since we split on ';')
            if op.is_empty() {
                return None;
            }

            let mut components = op.split_ascii_whitespace();

            if let Some(Ok(operation)) = components.next().map(OperationId::try_from_name) {
                Some(operation.create_instruction(components))
            } else {
                Some(Err(SicCliOpsError::InvalidScriptError(op.to_string())))
            }
        })
        .collect::<Result<_, _>>();

    std::env::remove_var("SIC_SCRIPT_WORKAROUND_UNQUOTE");

    ops
}

fn empty_ast(size_hint: Option<usize>) -> Vec<Instr> {
    let size = if let Some(size) = size_hint {
        size
    } else {
        128
    };

    Vec::with_capacity(size)
}

fn take_n<I: Iterator<Item = T>, T: AsRef<str> + Debug>(
    iter: &mut I,
    operation: OperationId,
) -> TResult<Vec<T>> {
    let args = iter
        .take(operation.takes_number_of_arguments())
        .collect::<Vec<_>>();

    if args.len() != operation.takes_number_of_arguments() {
        Err(SicCliOpsError::ExpectedArgumentForImageOperation(
            operation.as_str().to_string(),
            args.len(),
        ))
    } else {
        Ok(args)
    }
}

#[cfg(test)]
mod tests_script {
    use super::*;
    use sic_image_engine::engine::EnvItem;
    use sic_image_engine::engine::Instr;
    use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
    use sic_image_engine::wrapper::image_path::ImageFromPath;
    use sic_image_engine::ImgOp;

    #[parameterized(
        input = {
            "blur 1",
            "blur 1.0",
            "brighten 1",
            "brighten -1",
            "contrast 1",
            "contrast 1.0",
            "crop 0 0 10 10",
            "diff \"/my/path\"",
            "diff '/my/path'",
            "filter3x3 0 1 0 1 0 1 0 1 0",
            "flip-horizontal",
            "flip-vertical",
            "hue-rotate 90",
            "hue-rotate -90",
            "invert",
            // overlay
            "resize 100 100",
            "preserve-aspect-ratio true; resize 100 100",
            "sampling-filter catmullrom; resize 100 100",
            "rotate90",
            "rotate180",
            "rotate270",
            "unsharpen -0.7 1",
            "",
            ";;",
            "blur 1;",
        },
        expected = {
            vec![Instr::Operation(ImgOp::Blur(1.0))],
            vec![Instr::Operation(ImgOp::Blur(1.0))],
            vec![Instr::Operation(ImgOp::Brighten(1))],
            vec![Instr::Operation(ImgOp::Brighten(-1))],
            vec![Instr::Operation(ImgOp::Contrast(1.0))],
            vec![Instr::Operation(ImgOp::Contrast(1.0))],
            vec![Instr::Operation(ImgOp::Crop((0, 0, 10, 10)))],
            vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("/my/path".into())))],
            vec![Instr::Operation(ImgOp::Diff(ImageFromPath::new("/my/path".into())))],
            vec![Instr::Operation(ImgOp::Filter3x3([0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32, 1f32, 0f32]))],
            vec![Instr::Operation(ImgOp::FlipHorizontal)],
            vec![Instr::Operation(ImgOp::FlipVertical)],
            vec![Instr::Operation(ImgOp::HueRotate(90))],
            vec![Instr::Operation(ImgOp::HueRotate(-90))],
            vec![Instr::Operation(ImgOp::Invert)],
            // overlay
            vec![Instr::Operation(ImgOp::Resize((100, 100)))],
            vec![Instr::EnvAdd(EnvItem::PreserveAspectRatio(true)), Instr::Operation(ImgOp::Resize((100, 100)))],
            vec![Instr::EnvAdd(EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("catmullrom").unwrap())), Instr::Operation(ImgOp::Resize((100, 100)))],
            vec![Instr::Operation(ImgOp::Rotate90)],
            vec![Instr::Operation(ImgOp::Rotate180)],
            vec![Instr::Operation(ImgOp::Rotate270)],
            vec![Instr::Operation(ImgOp::Unsharpen((-0.7, 1)))],
            vec![],
            vec![],
            vec![Instr::Operation(ImgOp::Blur(1.0))],
        }
    )]
    fn parse_script(input: &str, expected: Vec<Instr>) {
        let parsed = create_image_ops_script(input.split(';').map(str::trim));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), expected);
    }

    #[cfg(feature = "imageproc-ops")]
    #[test]
    fn parse_script_with_imageproc_ops() {
        let input =
            "draw-text '<3' coord(10, 2) rgba(255, 0, 0, 255) size(14) font('./Lato-Regular.ttf')";
        let parsed = create_image_ops_script(input.split(';').map(str::trim));
        assert!(parsed.is_ok());
    }

    #[test]
    fn parse_script_expected_errors() {
        let input = "blur 15.7.; flipv";
        let parsed = create_image_ops_script(input.split(';').map(str::trim));

        assert!(parsed.is_err());
    }
}

#[cfg(test)]
mod tests_cli {
    use super::*;

    mod individual_args {
        use super::*;
        use sic_image_engine::engine::EnvItem;
        use sic_image_engine::wrapper::filter_type::FilterTypeWrap;
        use sic_image_engine::wrapper::image_path::ImageFromPath;
        use sic_image_engine::ImgOp;
        use sic_testing::setup_test_image;

        macro_rules! op {
            ($expr:expr) => {
                vec![Instr::Operation($expr)]
            };
        }

        macro_rules! ops {
            ($($expr:expr),*) => {{
                let mut vec = Vec::new();

                $(
                    vec.push(Instr::Operation($expr));
                )*

               vec
            }};
        }

        macro_rules! modifier {
            ($expr:expr) => {
                vec![Instr::EnvAdd($expr)]
            };
        }

        fn interweave(ops: &[&str]) -> Vec<String> {
            ops.iter()
                .map(|f| {
                    f.replace(
                        '▲',
                        &setup_test_image("aaa.png").to_string_lossy().to_string(),
                    )
                })
                .collect::<Vec<_>>()
        }

        ide!();

        #[parameterized(
            ops = {
                vec!["--blur", "1.0"],
                vec!["--brighten", "-1"],
                vec!["--contrast", "1.0"],
                vec!["--crop", "0", "1", "2", "3"],
                vec!["--diff", "▲"],
                vec!["--filter3x3", "1.0", "1.0", "1.0", "-1.0", "-1.0", "-1.0", "0.0", "0.0", "0.0"],
                vec!["--flip-horizontal"],
                vec!["--flip-vertical"],
                vec!["--grayscale"],
                vec!["--hue-rotate", "-1"],
                vec!["--invert"],
                vec!["--resize", "1", "1"],
                vec!["--preserve-aspect-ratio", "true"],
                vec!["--sampling-filter", "catmullrom"],
                vec!["--sampling-filter", "gaussian"],
                vec!["--sampling-filter", "lanczos3"],
                vec!["--sampling-filter", "nearest"],
                vec!["--sampling-filter", "triangle"],
                vec!["--rotate90"],
                vec!["--rotate180"],
                vec!["--rotate270"],
                vec!["--unsharpen", "-1.0", "-1"],
            },
            expected = {
                op![ImgOp::Blur(1.0)],
                op![ImgOp::Brighten(-1)],
                op![ImgOp::Contrast(1.0)],
                op![ImgOp::Crop((0, 1, 2, 3))],
                op![ImgOp::Diff(ImageFromPath::new(setup_test_image("aaa.png")))],
                op![ImgOp::Filter3x3([1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0])],
                op![ImgOp::FlipHorizontal],
                op![ImgOp::FlipVertical],
                op![ImgOp::GrayScale],
                op![ImgOp::HueRotate(-1)],
                op![ImgOp::Invert],
                op![ImgOp::Resize((1, 1))],
                modifier![EnvItem::PreserveAspectRatio(true)],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("catmullrom").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("gaussian").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("lanczos3").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("nearest").unwrap())],
                modifier![EnvItem::CustomSamplingFilter(FilterTypeWrap::try_from_str("triangle").unwrap())],
                op![ImgOp::Rotate90],
                op![ImgOp::Rotate180],
                op![ImgOp::Rotate270],
                op![ImgOp::Unsharpen((-1.0, -1))],
            },
        )]
        fn create_image_ops_t_sunny(ops: Vec<&str>, expected: Vec<Instr>) {
            let result = create_image_ops_cli(interweave(&ops));
            assert_eq!(result.unwrap(), expected);
        }

        #[cfg(feature = "imageproc-ops")]
        mod imageproc_ops_tests {
            use super::*;
            use sic_core::image::Rgba;
            use sic_image_engine::wrapper::draw_text_inner::DrawTextInner;
            use sic_image_engine::wrapper::font_options::{FontOptions, FontScale};
            use std::path::PathBuf;

            ide!();

            #[parameterized(
                ops = {
                    vec!["--draw-text", "my text", "coord(0, 1)", "rgba(10, 10, 255, 255)", "size(16.0)", r#"font("resources/font/Lato-Regular.ttf")"#],
                    vec!["--draw-text", "my text", "coord(0, 1)", "rgba(10, 10, 255, 255)", "size(16.0)", r#"font("resources/font/Lato-Regular()".ttf")"#],
                },
                expected = {
                    op![ImgOp::DrawText(DrawTextInner::new("my text".to_string(),
                        (0, 1),
                        FontOptions::new(
                        PathBuf::from("resources/font/Lato-Regular.ttf".to_string()),
                        Rgba([10, 10, 255, 255]),
                        FontScale::Uniform(16.0))))],
                    op![ImgOp::DrawText(DrawTextInner::new("my text".to_string(),
                        (0, 1),
                        FontOptions::new(
                        PathBuf::from("resources/font/Lato-Regular()\".ttf".to_string()),
                        Rgba([10, 10, 255, 255]),
                        FontScale::Uniform(16.0))))]
                }
            )]
            fn create_image_ops_t_sunny_imageproc_ops(ops: Vec<&str>, expected: Vec<Instr>) {
                let result = create_image_ops_cli(interweave(&ops));

                assert_eq!(result.unwrap(), expected);
            }
        }

        #[test]
        fn combined() {
            let input = vec![
                "--blur",
                "1.0",
                "--brighten",
                "-1",
                "--contrast",
                "1.0",
                "--crop",
                "0",
                "1",
                "2",
                "3",
                "--diff",
                &setup_test_image("aaa.png").to_string_lossy().to_string(),
                "--filter3x3",
                "1.0",
                "1.0",
                "1.0",
                "-1.0",
                "-1.0",
                "-1.0",
                "0.0",
                "0.0",
                "0.0",
                "--flip-horizontal",
            ]
            .iter()
            .map(|v| (*v).to_string())
            .collect::<Vec<_>>();

            let expected = ops![
                ImgOp::Blur(1.0),
                ImgOp::Brighten(-1),
                ImgOp::Contrast(1.0),
                ImgOp::Crop((0, 1, 2, 3)),
                ImgOp::Diff(ImageFromPath::new(setup_test_image("aaa.png"))),
                ImgOp::Filter3x3([1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 0.0, 0.0, 0.0]),
                ImgOp::FlipHorizontal
            ];

            assert_eq!(create_image_ops_cli(input).unwrap(), expected);
        }

        #[parameterized(
            ops = {
                vec!["--blur", "A"],
                vec!["--brighten", "-1.0"],
                vec!["--contrast", ""],
                vec!["--crop", "--crop", "0", "1", "2", "3"],
                vec!["--diff"],
                vec!["--filter3x3", "[", "1.0", "1.0", "1.0", "-1.0", "-1.0", "-1.0", "0.0", "0.0", "0.0", "]"],
                vec!["--hue-rotate", "-100.8"],
                vec!["--resize", "1", "1", "--crop"],
                vec!["--preserve-aspect-ratio", "yes"],
                vec!["--sampling-filter", "tri"],
                vec!["--sampling-filter", ""],
                vec!["--unsharpen", "-1.0", "-1.0"],
            }
        )]
        fn create_image_ops_t_expected_failure(ops: Vec<&str>) {
            let result = create_image_ops_cli(interweave(&ops));
            assert!(result.is_err());
        }
    }
}
