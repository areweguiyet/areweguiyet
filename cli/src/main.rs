#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate hyper;
extern crate hyper_tls;

#[macro_use]
extern crate tera;

use hyper::Client;
use hyper::rt::{self, Future, Stream};
use hyper_tls::HttpsConnector;

use serde::de::DeserializeOwned;

use std::path::Path;
use std::io::Write;
use std::fs::File;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::channel;
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

#[derive(Serialize, Deserialize)]
struct Cache {
    crates_io: HashMap<String, CrateResponse>
}

impl Default for Cache {
    fn default() -> Cache {
        Cache {
            crates_io: HashMap::new()
        }
    }
}

// TODO: This should only have a "crate" key but that's a keyword in rust...
type CratesResponse = HashMap<String, serde_json::Value>;

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

fn compile_templates_and_write<P: AsRef<Path>>(awgy: &AreWeGuiYet, out_path: P) {
    let tera = compile_templates!("../site/**/*.tera.html");
    let index = tera.render("base.tera.html", awgy)
        .expect("Failed to render templates");

    let mut out = File::create(out_path)
        .expect("Failed to create output file");
    out.write_all(index.as_bytes())
        .expect("Failed to write everything to the output file");
}

/// Makes an API request to crates.io if there is no custom repo specified for the crate
fn get_crate_info(cache: &mut Cache, krate: &mut Crate) {
    if krate.repo.is_some() {
        return;
    }

    let raw_url = format!("https://crates.io/api/v1/crates/{}", krate.name);
    let url = raw_url.parse::<hyper::Uri>()
        .unwrap();

    let res = cache.crates_io.entry(krate.name.clone())
        .or_insert_with(|| {
            println!("Cache miss. Requesting data for {}", &krate.name);
            // TODO: Clean this up, simplify it, and make it as rusty as possible
            // It's quite wasteful I think as it is right now, but I don't know hyper/tokio very well!
            let (sender, receiver) = channel();

            rt::run(rt::lazy(move || {
                let https = HttpsConnector::new(4)
                    .expect("TLS initialization failed");
                let client = Client::builder()
                    .build::<_, hyper::Body>(https);

                client
                    // Fetch the url...
                    .get(url)
                    .map(|res| -> CratesResponse {
                        let res = res.into_body()
                            .concat2()
                            .wait()
                            .unwrap()
                            .into_bytes()
                            .to_vec();
                        //println!("Got response {}", String::from_utf8_lossy(res.as_slice()));
                        serde_json::from_reader(res.as_slice())
                            .expect("Crates IO did not return valid JSON")
                    })
                    .map_err(|err| eprintln!("Failed to make request with error {}", err))
                    .map(|mut crate_json|
                        // TODO: Error message when crate is missing
                        match crate_json.remove("crate") {
                            Some(crate_json) => serde_json::from_value(crate_json)
                                .expect("Failed to parse relevant data from the crate"),
                            None => Default::default()
                        }
                    )
                    .map(move |crate_data: CrateResponse|
                        sender.send(crate_data)
                            .unwrap()
                    )
            }));

            receiver.recv()
                .unwrap()
        });

    let CrateResponse {
        description,
        repository,
        documentation,
    } = res.clone();

    krate.crates_io = Some(raw_url);
    krate.repo = krate.repo.clone().or(repository);
    krate.docs = krate.docs.clone().or(documentation);
    krate.description = krate.description.clone().or(description);
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
