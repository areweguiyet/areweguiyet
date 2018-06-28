#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate reqwest;

#[macro_use]
extern crate tera;

use serde::de::DeserializeOwned;

use std::path::Path;
use std::io::Write;
use std::fs::File;
use std::collections::{HashMap, HashSet};
use std::error::Error;

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
    crates_io: Option<String>,
    repo: Option<String>,
    description: Option<String>,
    docs: Option<String>,
    tags: Vec<String>,
}

/// Stores parsed raw requests data from any services we query (like crates.io or GitHub).
#[derive(Serialize, Deserialize)]
struct Cache {
    crates_io: HashMap<String, Option<CrateResponse>>
}

impl Cache {
    fn get_crates_io(&mut self, name: &str) -> Result<&Option<CrateResponse>, Box<Error>> {
        // We can perhaps make this better with NLL?
        let url = crates_io_url(name);
        if !self.crates_io.contains_key(name) {
            println!("Cache miss. Requesting data for {}", name);
            let mut res = reqwest::get(&url)?;
            let parsed: Option<CratesResponse> = match res.status() {
                reqwest::StatusCode::Ok => res.json()?,
                reqwest::StatusCode::NotFound => None,
                _ => return Err("Unknown request error".into()),
            };
            self.crates_io.insert(name.to_string(), parsed.map(|x| x.krate));
        }
        Ok(&self.crates_io[name])
    }
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            crates_io: HashMap::new()
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CratesResponse {
    #[serde(rename = "crate")]
    krate: CrateResponse,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CrateResponse {
    description: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
}

fn parse_json_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, Box<Error>> {
    let f = File::open(path)?;
    Ok(serde_json::from_reader(f)?)
}

fn write_cache<P: AsRef<Path>>(cache: &Cache, path: P) {
    let out = match File::create(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Nonfatal: Failed to create cache file.\nError: {}", e);
            return;
        }
    };

    serde_json::to_writer_pretty(out, cache)
        .expect("Failed to write the cache");
}

/// Compiles the tera templates from a hard coded path (the site directory).
fn compile_templates_and_write<P: AsRef<Path>>(awgy: &AreWeGuiYet, out_path: P) {
    let tera = compile_templates!("../site/**/*.tera.html");
    let index = tera.render("base.tera.html", awgy)
        .expect("Failed to render templates");

    let mut out = File::create(out_path)
        .expect("Failed to create output file");
    out.write_all(index.as_bytes())
        .expect("Failed to write everything to the output file");
}

/// Uses data from crates.io if there is no custom repo specified for the crate.
///
/// No fields will be overwritten if they are already specified.
fn get_crate_info(cache: &mut Cache, krate: &mut Crate) {
    if krate.repo.is_some() {
        return;
    }

    let res = cache.get_crates_io(&krate.name)
        .expect("Failed to fetch from Crates.io");

    if let Some(res) = res {
        let CrateResponse {
            description,
            repository,
            documentation,
        } = res.clone();

        let url = crates_io_url(&krate.name);
        krate.crates_io = Some(url);
        krate.repo = krate.repo.clone().or(repository);
        krate.docs = krate.docs.clone().or(documentation);
        krate.description = krate.description.clone().or(description);
    }

}

fn crates_io_url(crate_name: &str) -> String {
    format!("https://crates.io/api/v1/crates/{}", crate_name)
}

fn main() {
    // TODO: Error handling! ;^)
    let mut crates: Vec<Crate> = parse_json_file("../ecosystem.json")
        .expect("Failed to parse ecosystem.json");
    let mut tags: HashMap<String, Option<String>> = parse_json_file("../ecosystem_tags.json")
        .expect("Failed to parse ecosystem_tags.json");
    let mut cache: Cache = parse_json_file("../cache.json")
        .unwrap_or_else(|_| Default::default());

    // TODO: Lack of non-lexical lifetimes means we need a special scope for used_tags
    // (used_tags borrows crates)
    {
        // figure out which tags we used and generate missing crate information
        let mut used_tags = HashSet::new();
        for krate in &mut crates {
            get_crate_info(&mut cache, krate);
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

    // update the cache
    write_cache(&cache, "../cache.json");

    compile_templates_and_write(&awgy, "../docs/index.html");
}
