#[macro_use]
pub mod app;

extern crate strum;
#[macro_use]
extern crate strum_macros;

pub mod cli_parse;

pub fn get_tool_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}
