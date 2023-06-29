#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate reqwest;

extern crate tera;

extern crate clap;

mod cli;
mod newsfeed;

fn main() {
    cli::execute_cli();
}
