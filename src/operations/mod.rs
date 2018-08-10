use image::DynamicImage;
use pest::iterators::Pairs;
#[allow(unused_imports)]
use pest::Parser;

mod apply_operations;

// ensure grammar refreshes on compile
const _GRAMMAR: &str = include_str!("grammar.pest");
const PARSER_RULE: Rule = Rule::main;

#[derive(Parser)]
#[grammar = "operations/grammar.pest"]
struct SICParser;

#[derive(Debug, PartialEq)]
pub enum Operation {
    Blur(u32),
    FlipHorizontal,
    FlipVertical,
    Resize(u32, u32),
}

pub type Operations = Vec<Operation>;

pub fn parse_and_apply_script(image: DynamicImage, script: &str) -> Result<DynamicImage, String> {
    let parsed_script = SICParser::parse(PARSER_RULE, script);
    let rule_pairs: Pairs<Rule> = parsed_script
        .unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));

    let operations: Result<Operations, String> = parse_image_operations(rule_pairs);

    match operations {
        Ok(ops) => apply_operations::apply_operations_on_image(image, &ops),
        Err(err) => Err(err)
    }
}

// This function has been reworked to include error handling.
// Some errors should never happen because the parser generated by Pest should catch them already.
// Nevertheless, to guard against mistakes, error handling is now included.
// >> This function has several unwraps which should not be able to fail.
// >> The reason for this is that the pest grammar would not allow SICParse::parse() to succeed.
// >> This does assume that the grammar is correct... ;).
// >> Not grammatical constraints should be tested for and that's why this function
// >>  does return a Result.
// >> Perhaps not unwrapping on every pair could be implemented in the future, but for now,
// >>  the code will use unwrap to not become to awkward.
// >> The Pest book describes usage of unwrap as idiomatic [1].
//
// >> Possible missteps:
// >> - u32 out of bounds (the grammar describes "infinite unsigned integers")
// >> - repetition of sub rules (not known at compile time (this should be checked)
//
//
// [1] https://pest-parser.github.io/book/parser_api.html
pub fn parse_image_operations(pairs: Pairs<Rule>) -> Result<Operations, String> {
    pairs
        .map(|pair| match pair.as_rule() {
            Rule::blur => {
                let u_int_text = pair
                    .into_inner()
                    .next()
                    .ok_or_else(|| "Unable to parse and capture `blur` value.".to_string())
                    .map(|val| val.as_str());

                let u_int = u_int_text
                    .and_then(|it: &str| it.parse::<u32>()
                        .map_err(|e| e.to_string()));

                u_int.map(|u| Operation::Blur(u))
            },
            Rule::flip_horizontal => Ok(Operation::FlipHorizontal),
            Rule::flip_vertical => Ok(Operation::FlipVertical),
            Rule::resize => {
                let mut inner = pair.into_inner();

                let x_text = inner
                    .next()
                    .ok_or_else(|| "Unable to parse `resize <x> <y>`".to_string())
                    .map(|val| val.as_str());

                let x = x_text
                    .and_then(|it: &str| it.parse::<u32>()
                        .map_err(|e| e.to_string()));


                let y_text = inner
                    .next()
                    .ok_or_else(|| "Unable to parse `resize <x> <y>`".to_string())
                    .map(|val| val.as_str());

                let y = y_text
                    .and_then(|it: &str| it.parse::<u32>()
                        .map_err(|e| e.to_string()));

                x.and_then(|ux| {
                    y.map(|uy| {
                        Operation::Resize(ux, uy)
                    })
                })
            },
            _ => Err("Parse failed: Operation doesn't exist".to_string()),
        }).collect::<Result<Operations, String>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    const _TEST_IMAGE_PATH: &str = "resources/unsplash_763569_cropped.jpg";

    fn _setup() -> DynamicImage {
        use std::path::Path;
        image::open(&Path::new(_TEST_IMAGE_PATH)).unwrap()
    }

    fn _manual_inspection(img: &DynamicImage, path: &str) {
        if !cfg!(feature = "dont-run-on-ci") {
            let _ = img.save(path);
        }
    }

    #[test]
    fn test_multi_parse_and_apply_script() {
        let image = _setup();
        let script: &str = "flip_horizontal; resize 100 100; blur 3;";

        let result = parse_and_apply_script(image, script);

        assert!(result.is_ok());

        let _ = _manual_inspection(&result.unwrap(), "target/parse_util_apply_all.png");
    }

    #[test]
    fn test_blur_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "blur 15;")
            .unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(Ok(vec![Operation::Blur(15)]), parse_image_operations(pairs));
    }

    #[test]
    fn test_flip_horizontal_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "flip_horizontal;")
            .unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![Operation::FlipHorizontal]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_flip_vertical_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "flip_vertical;")
            .unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(Ok(vec![Operation::FlipVertical]), parse_image_operations(pairs));
    }

    #[test]
    fn test_resize_single_stmt_parse_correct() {
        let pairs = SICParser::parse(Rule::main, "resize 99 88;")
            .unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![Operation::Resize(99, 88)]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_multi_stmt_parse_correct() {
        let pairs = SICParser::parse(
            Rule::main,
            "blur 10;flip_horizontal;flip_vertical;resize 100 200;",
        ).unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![
                Operation::Blur(10),
                Operation::FlipHorizontal,
                Operation::FlipVertical,
                Operation::Resize(100, 200)
            ]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_multi_stmt_parse_diff_order_correct() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip_horizontal;flip_vertical;resize 100 200;blur 10;",
        ).unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![
                Operation::FlipHorizontal,
                Operation::FlipVertical,
                Operation::Resize(100, 200),
                Operation::Blur(10)
            ]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_multi_whitespace() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip_horizontal; flip_vertical; resize 100 200; blur 10;",
        ).unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![
                Operation::FlipHorizontal,
                Operation::FlipVertical,
                Operation::Resize(100, 200),
                Operation::Blur(10)
            ]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_multi_whitespace_2() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip_horizontal    ; flip_vertical   ;   \t\t resize 100 200; blur 10;",
        ).unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![
                Operation::FlipHorizontal,
                Operation::FlipVertical,
                Operation::Resize(100, 200),
                Operation::Blur(10)
            ]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_multi_whitespace_3() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip_horizontal;\nflip_vertical;\nresize 100 200;\n\tblur 10;",
        ).unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![
                Operation::FlipHorizontal,
                Operation::FlipVertical,
                Operation::Resize(100, 200),
                Operation::Blur(10)
            ]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_multi_should_no_longer_end_with_sep() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip_horizontal; flip_vertical; resize 100 200; blur 10",
        ).unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![
                Operation::FlipHorizontal,
                Operation::FlipVertical,
                Operation::Resize(100, 200),
                Operation::Blur(10)
            ]),
            parse_image_operations(pairs)
        );
    }

    #[test]
    fn test_multi_sep_optional() {
        let pairs = SICParser::parse(
            Rule::main,
            "flip_horizontal flip_vertical; resize 100 200 blur 10",
        ).unwrap_or_else(|e| panic!("Unable to parse sic image operations script: {:?}", e));
        assert_eq!(
            Ok(vec![
                Operation::FlipHorizontal,
                Operation::FlipVertical,
                Operation::Resize(100, 200),
                Operation::Blur(10)
            ]),
            parse_image_operations(pairs)
        );
    }
}
