#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate reqwest;

#[macro_use]
extern crate tera;

extern crate clap;

mod newsfeed;
mod cli;

fn main() {
    cli::execute_cli();
    return;
}
