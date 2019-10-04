use crate::cli_parse::prototype;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct TestFile {
    image_path: PathBuf,
    cases: Vec<Case>,
}

#[derive(Debug)]
struct Case {
    line: String,
    expected_outcome: Outcome,
}

#[derive(Debug, Clone, Copy)]
enum Outcome {
    Success,
    Failure,
}

fn load_test_files() -> Vec<TestFile> {
    let top = &[env!("CARGO_MANIFEST_DIR"), "/resources/tests-parser/"].concat();
    let top = Path::new(top);

    std::fs::read_dir(top)
        .unwrap()
        .into_iter()
        .filter_map(|path| {
            if let Some(ext) = path.as_ref().unwrap().path().extension() {
                if ext == "sic" {
                    let cases = path.as_ref().unwrap().path();
                    return Some(load_test_file(&cases));
                }
            }
            None
        })
        .collect::<Vec<_>>()
}

fn load_test_file(path: &Path) -> TestFile {
    let lines = std::fs::read_to_string(path).map(|lines| {
        lines
            .lines()
            .filter_map(|line| {
                if line.starts_with("#") {
                    None
                } else if line.starts_with("@") {
                    Some(Case {
                        line: line[1..].to_string(), // lets hope we don't ever need to deal with wonky unicode here, because we slice on bytes <3
                        expected_outcome: Outcome::Failure,
                    })
                } else {
                    Some(Case {
                        line: String::from(line),
                        expected_outcome: Outcome::Success,
                    })
                }
            })
            .collect::<Vec<Case>>()
    });

    return TestFile {
        image_path: path.with_extension("jpg"),
        cases: lines.unwrap(),
    };
}

fn run_test(test_file: &TestFile) {
    for case in &test_file.cases {
        let input = case
            .line
            .split_ascii_whitespace()
            .map(String::from)
            .collect::<Vec<_>>()
            .into_iter();

        let out = prototype(input);

        match case.expected_outcome {
            Outcome::Success => {
                assert!(out.is_ok());
                let out = out.unwrap();
                println!("optree: {:?}\n", out);
                assert!(out.len() > 0);
            }
            Outcome::Failure => assert!(out.is_err()),
        }
    }
}

#[test]
fn all() {
    let vec = load_test_files();

    const DIVIDER: &str =
        "================================================================================";

    for (i, test) in vec.iter().enumerate() {
        let name = test.image_path.file_name().unwrap();
        let name = name.to_str().unwrap();
        let name: Vec<&str> = name.split(".").collect();

        println!("\n{}\n* test#: {}, name: {:?}\n", DIVIDER, i, name[0]);
        run_test(test);
    }
}

#[test]
fn multi_arg() {
    let path = &[
        env!("CARGO_MANIFEST_DIR"),
        "/resources/tests-parser/arg_modifiers.sic",
    ]
    .concat();
    let path = Path::new(path);
    let test_file = load_test_file(path);
    run_test(&test_file);
}
