use clap::Command;
use crates_io_api::CrateResponse;
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

const USER_AGENT: &str = "areweguiyet_cli (areweguiyet.com)";

fn cli() -> Command {
    Command::new("AreWeGuiYet CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .about("CLI for fetching data from various sources for the AreWeGuiYet website")
        .subcommand(Command::new("clean").about("Remove the data"))
        .subcommand(Command::new("fetch").about("Fetch new data"))
        .subcommand(
            Command::new("update-images")
                .about("Update Rust images from the rust-lang.org repository"),
        )
}

pub fn execute_cli() {
    let matches = cli().get_matches();

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    match matches.subcommand() {
        Some(("clean", _)) => ExternalData::clean(root),
        Some(("fetch", _)) => fetch(root),
        Some(("update-images", _)) => update_images(root),
        _ => unreachable!(),
    }
}

/// All the info in the ecosystem file
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Ecosystem {
    #[serde(rename = "crate")]
    crates: HashMap<String, Crate>,
}

impl Ecosystem {
    fn load(root: &Path) -> Self {
        let s =
            fs::read_to_string(root.join("ecosystem.toml")).expect("failed reading ecosystem.toml");
        toml::from_str(&s).unwrap_or_else(|err| panic!("failed parsing ecosystem.toml: {err}"))
    }
}

/// Crate info in ecosystem file
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Crate {
    name: Option<String>,
    /// Should be either missing or true; implied to be false
    #[serde(default)]
    #[serde(rename = "skip-crates-io")]
    skip_crates_io: bool,
    repo: Option<String>,
    description: Option<String>,
    docs: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// Data from various sources that we're currently storing.
#[derive(Serialize, Deserialize, Default)]
struct ExternalData {
    /// A map of crate IDs to crate data fetched from crates.io.
    crates_io: BTreeMap<String, CrateResponse>,
}

impl ExternalData {
    const FILE: &str = "target/external_data.json";

    fn clean(root: &Path) {
        // Remove the data, and ignore if the file was not found
        if let Err(err) = fs::remove_file(root.join(Self::FILE)) {
            if err.kind() != io::ErrorKind::NotFound {
                panic!("failed to remove the external data file: {err}");
            }
        };
        println!("External data file removed.");
    }

    fn load(root: &Path) -> Self {
        // Remove the data, and return Default if the file was not found
        match fs::read_to_string(root.join(Self::FILE)) {
            Ok(s) => serde_json::from_str(&s).expect("failed parsing external data"),
            Err(err) => {
                if err.kind() == io::ErrorKind::NotFound {
                    Default::default()
                } else {
                    panic!("failed reading external data file: {err}");
                }
            }
        }
    }

    fn write(&self, root: &Path) {
        let s = serde_json::to_string_pretty(self).expect("failed to serialize external data");
        fs::write(root.join(Self::FILE), s).expect("failed to write data file");
    }
}

fn fetch(root: &Path) {
    let mut data = ExternalData::load(root);

    // Add crate information from crates.io
    let ecosystem = Ecosystem::load(root);
    println!("Found {} crates.", ecosystem.crates.len());

    let client = crates_io_api::SyncClient::new(
        USER_AGENT,
        // Use the recommended rate limit
        Duration::from_millis(1000),
    )
    .expect("failed initializing crates.io client");

    for (crate_id, krate) in &ecosystem.crates {
        if krate.skip_crates_io {
            continue;
        }

        if !data.crates_io.contains_key(crate_id) {
            print!("Requesting crates.io data for {crate_id}... ");
            let response = client
                .get_crate(crate_id)
                .unwrap_or_else(|err| panic!("could not find crate {crate_id}: {err}"));
            data.crates_io.insert(crate_id.to_string(), response);
            println!("done.");
        }

        check_compiled_crate(crate_id, krate, &data.crates_io[crate_id]);
    }

    data.write(root);
    println!("External data fetched.");
}

/// Issues errors if the data from crates io is the same as the local data.
fn check_compiled_crate(crate_id: &str, krate: &Crate, crates_io: &CrateResponse) {
    let crates_io_api::Crate {
        repository,
        description,
        documentation,
        ..
    } = crates_io.crate_data.clone();

    if krate.repo.is_some() && krate.repo == repository {
        panic!(
            "Please remove {crate_id}'s repo in ecosystem.toml since it duplicates the value on crates.io",
        );
    }

    if krate.description.is_some() && krate.description == description {
        panic!(
            "Please remove {crate_id}'s description in ecosystem.toml since it duplicates the value on crates.io",
        );
    }

    if krate.docs.is_some() && krate.docs == documentation {
        panic!(
            "Please remove {crate_id}'s docs in ecosystem.toml since it duplicates the value on crates.io",
        );
    }
}

fn update_images(root: &Path) {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("failed initializing image downloader client");

    let files = [
        "apple-touch-icon.png",
        "favicon-16x16.png",
        "favicon-32x32.png",
        "favicon.svg",
        "favicon.ico",
    ];
    for file in files {
        let url = format!("https://raw.githubusercontent.com/rust-lang/www.rust-lang.org/master/static/images/{file}");
        let response = client.get(url).send().expect("failed fetching image");
        response
            .error_for_status_ref()
            .expect("got an error while fetching image");
        fs::write(
            root.join("static/images").join(file),
            response.bytes().expect("no body"),
        )
        .expect("failed writing image");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        cli().debug_assert();
    }
}
