#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate tera;

use serde::de::DeserializeOwned;

use std::path::Path;
use std::io::Write;
use std::fs::File;
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize)]
struct AreWeGuiYet {
    crates: Vec<Crate>,
    /// Collection of tags.
    ///
    /// Some tags may have descriptions.
    tags: HashMap<String, Option<String>>,
}

#[derive(Serialize, Deserialize)]
struct Crate {
    name: String,
    // TODO: Pull this from crates.io/github/or a repo...
    description: String,
    tags: Vec<String>,
}

fn parse_json_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> serde_json::Result<T> {
    let f = File::open(path)
        .expect("Internal error: Failed to find file specified");
    serde_json::from_reader(f)
}

fn main() {
    // TODO: Error handling
    let crates: Vec<Crate> = parse_json_file("../ecosystem.json")
        .expect("Failed to parse ecosystem.json");
    let mut tags: HashMap<String, Option<String>> = parse_json_file("../ecosystem_tags.json")
        .expect("Failed to parse ecosystem_tags.json");

    // figure out which tags we used
    // TODO: Lack of non-lexical lifetimes means we need a special scope for used_tags
    // (used_tags borrows crates)
    {
        let mut used_tags = HashSet::new();
        for krate in &crates {
            used_tags.extend(krate.tags.iter())
        }

        // issue a warning if there are unsused tags in ecosystem_tags.json
        for (k, _) in &tags {
            if !used_tags.contains(k) {
                println!("Tag \"{}\" is not used to describe any crate", k);
            }
        }

        // merge description-less used tags into ecosystem_tags
        for k in used_tags {
            tags.insert(k.to_string(), None);
        }
    }

    let awgy = AreWeGuiYet {
        crates,
        tags,
    };

    let tera = compile_templates!("../site/**/*.tera.html");
    let index = tera.render("base.tera.html", &awgy)
        .expect("Failed to render templates");

    let mut out = File::create("../docs/index.html")
        .expect("Failed to create output file");
    out.write_all(index.as_bytes())
        .expect("Failed to write everything to the output file");
}
