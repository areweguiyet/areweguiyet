use newsfeed::NewsfeedEntry;

use serde::de::DeserializeOwned;

use clap::{App, SubCommand, Arg, AppSettings, ArgGroup};

use std::path::Path;
use std::io::Write;
use std::fs::File;
use std::collections::{HashMap, HashSet};
use std::error::Error;

use std::fs;
use std::io;

const NEWSFEED: &str = "../newsfeed.json";
const ECOSYSTEM: &str = "../ecosystem.json";
const COMPILED_ECOSYSTEM: &str = "../docs/compiled_ecosystem.json";
const ECOSYSTEM_TAGS: &str = "../ecosystem_tags.json";

const TEMPLATE_SOURCE_GLOB: &str = "../site/**/*.tera.html";
const INDEX_HTML_OUTPUT_PATH: &str = "../docs/index.html";
const INDEX_HTML_TEMPLATE_NAME: &str = "base.tera.html";

const CACHE_FILE: &str = "../cache.json";
const CACHE_FILE_DELETION_FAILED: &str = "Failed to remove the cache file. Try deleting it \
    manually and running without the clean option.";

/// The arguments needed for Tera to render the template.
#[derive(Serialize, Deserialize)]
struct AreWeGuiYetTemplateArgs {
    crates: Vec<Crate>,
    /// Collection of tags.
    ///
    /// Some tags may have descriptions.
    tags: HashMap<String, Option<String>>,
    newsfeed: Vec<NewsfeedEntry>,
}

/// Crate info in ecosystem.json
#[derive(Serialize, Deserialize)]
struct Crate {
    name: String,
    /// Should be either missing or true; implied to be false
    #[serde(default)]
    skip_crates_io: bool,
    repo: Option<String>,
    description: Option<String>,
    docs: Option<String>,
    tags: Vec<String>,
}

/// Crate info that gets put into the compiled ecosystem file
#[derive(Serialize, Deserialize)]
struct CompiledCrate {
    // name: String, // Compiled Crates are stored in a HashMap, no longer need this
    crates_io: Option<String>,
    repo: Option<String>,
    description: Option<String>,
    docs: Option<String>,
    tags: Vec<String>,
}

/// Stores parsed raw requests data from any services we query (like crates.io or GitHub).
#[derive(Serialize, Deserialize)]
struct Cache {
    crates_io: HashMap<String, Option<CratesIoCrateResponse>>
}

impl Cache {
    /// Attempt to load cache from path; creates empty cache on failure
    fn new(path: &str) -> Self {
        parse_json_file(path)
            .unwrap_or_else(|_| Default::default())
    }

    /// Get crate meta data from crates io API, and cache the result
    fn get_crates_io(&mut self, name: &str) -> Result<&Option<CratesIoCrateResponse>, Box<Error>> {
        // We can perhaps make this better with NLL?
        if !self.crates_io.contains_key(name) {
            println!("Cache miss. Requesting data for {}", name);
            let url = crates_io_url(name);
            let mut res = reqwest::get(&url)?;
            let parsed: Option<CratesIoEnvelopeResponse> = match res.status() {
                reqwest::StatusCode::Ok => res.json()?,
                reqwest::StatusCode::NotFound => None,
                _ => return Err("Unknown request error".into()),
            };
            self.crates_io.insert(name.to_string(), parsed.map(|x| x.krate));
        }
        Ok(&self.crates_io[name])
    }

    /// Save the cache to disk at path
    fn write_cache(&self, path: &str) -> Result<(), Box<Error>> {
        let out = File::create(path)?;

        // This is fatal because running the app again will cause it fail as the cache file exists
        // but may contain garbage/be empty
        serde_json::to_writer_pretty(out, self)
            .expect("Failed to write the cache file.");

        Ok(())
    }

    /// Removes the cache file and clears the cache data
    fn remove_cache(&mut self, path: &str) -> Result<(), io::Error> {
        self.crates_io.clear();
        // remove the cache (NotFound errors are fine)
        fs::remove_file(path)
            .or_else(|err|
                if err.kind() != io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(err)
                })
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
struct CratesIoEnvelopeResponse {
    #[serde(rename = "crate")]
    krate: CratesIoCrateResponse,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CratesIoCrateResponse {
    description: Option<String>,
    repository: Option<String>,
    documentation: Option<String>,
}

pub fn execute_cli() {
    let matches = App::new("Areweguiyet CLI")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("CLI for extending areweguiyet website")
        .subcommand(SubCommand::with_name("publish")
            .about("Publishes generated HTML to docs directory. Fork the repo, push the resulting \
                  changes, and then open a PR on Github to share your changes!")
            .arg(Arg::with_name("clean")
                .long("clean")
                .help("Force refreshes the cache, making new network requests"))
            .arg(Arg::with_name("verify-only")
                .long("verify-only")
                .help("Run all normal checks before publishing without generating HTML.")))
        .subcommand(SubCommand::with_name("framework")
            .about("Adds a new GUI crate or framework to ecosystem.json."))
        .subcommand(SubCommand::with_name("news")
            .about("Adds a new news post from either a link or a markdown file.")
            .arg(Arg::with_name("link")
                .long("link")
                .short("l")
                .help("Adds a new news post from a link to another website"))
            .arg(Arg::with_name("post")
                .long("post")
                .short("p")
                .help("Creates a new news post hosted on Areweguiyet"))
            .group(ArgGroup::with_name("newsfeed_type")
                .args(&["post", "link"])
                .required(true)))
        .get_matches();

    match matches.subcommand() {
        ("publish", args) => {
            let (clean, verify_only) = match args {
                Some(args) => (
                    args.is_present("clean"),
                    args.is_present("verify-only")
                ),
                None => (false, false),
            };

            publish(clean, verify_only);
        },
        ("framework", _) => {
            unimplemented!();
        },
        ("news", _) => {
            unimplemented!();
        },
        _ => unreachable!(),
    }
}

/// Compile ecosystem info, cache result, and generate warnings
fn publish(clean: bool, verify_only: bool) {
    let mut cache = Cache::new(CACHE_FILE);

    if clean {
        cache.remove_cache(CACHE_FILE)
            .expect(CACHE_FILE_DELETION_FAILED);
        println!("Cache file removed.");
    }

    // Load all the information we need
    let mut crates: Vec<Crate> = parse_json_file(ECOSYSTEM)
        .expect("Failed to parse ecosystem.json");
    let mut tags: HashMap<String, Option<String>> = parse_json_file(ECOSYSTEM_TAGS)
        .expect("Failed to parse ecosystem_tags.json");
    let newsfeed: Vec<NewsfeedEntry> = parse_json_file(NEWSFEED)
        .expect("Failed to parse newsfeed.json");

    println!("Found {} crates.", crates.len());

    let mut errors = Vec::new();

    // TODO: Verify that tag names and descriptions don't contain unwanted characters.
    // TODO: Lack of non-lexical lifetimes means we need a special scope for used_tags
    // (used_tags borrows crates, but crates needs to be movable outside this block)
    {
        // verify that every tag in the tags file is actually used
        let mut used_tags = HashSet::new();

        for krate in &mut crates {
            used_tags.extend(krate.tags.iter())
        }

        // issue a warning if there are unsused tags in ecosystem_tags.json
        for (k, _) in &tags {
            if !used_tags.contains(k) {
                errors.push(format!("Tag \"{}\" is not used to describe any crate", k));
            }
        }

        // merge description-less used tags into ecosystem_tags
        for k in used_tags {
            tags.entry(k.to_string())
                .or_insert(None);
        }
    }

    // merge missing crate information from crates io
    let mut compiled_ecosystem = HashMap::new();
    for krate in &mut crates {
        let compiled_crate = get_crate_info(krate, &mut cache, &mut errors);
        compiled_ecosystem.insert(krate.name.clone(), compiled_crate);
    }

    // compile the template
    let awgy = AreWeGuiYetTemplateArgs {
        crates,
        tags,
        newsfeed,
    };

    // update the cache
    match cache.write_cache("../cache.json") {
        Ok(_) => println!("Cache updated."),
        Err(_) => println!("Nonfatal: Failed to write the cache"),
    }

    let tera = compile_templates!(TEMPLATE_SOURCE_GLOB);
    // Render the template and remove newlines so people don't accidentally edit the compiled HTML
    // (we could actually minify it too)
    let index = tera.render(INDEX_HTML_TEMPLATE_NAME, &awgy)
        .expect("Failed to render templates")
        .replace("\r\n", " ")
        .replace("\n", " ");

    println!("Successfully rendered templates.");

    if errors.len() > 0 {
        eprintln!("The following issues are preventing HTML generation:");
        for i in &errors {
            println!("\t{}", i);
        }
        panic!("Failed to generate site.");
    }

    if !verify_only {
        let mut out_compiled_ecosystem = File::create(COMPILED_ECOSYSTEM)
            .expect("Failed to create compiled ecosystem file");
        serde_json::to_writer(&mut out_compiled_ecosystem, &compiled_ecosystem)
            .expect("Failed to write the compiled ecosystem to the output file");

        let mut out_index = File::create(INDEX_HTML_OUTPUT_PATH)
            .expect("Failed to create index output file");
        out_index.write_all(index.as_bytes())
            .expect("Failed to write the index to the output file");

        println!("Site written to disk.");
    } else {
        println!("Skipping writing the site.");
    }
}

/// Merge saved data with data from crates io (if the crate is on crates io).
    ///
    /// No fields will be overwritten if they are already specified.
    ///
    /// Issues errors if the data from crates io is the same as the local data.
fn get_crate_info(krate: &Crate, cache: &mut Cache, errors: &mut Vec<String>) -> CompiledCrate {
    let crates_io;

    if !krate.skip_crates_io {
        let res = cache.get_crates_io(&krate.name)
            .expect("Failed to fetch from Crates.io");

        if let Some(res) = res {
            let url = crates_io_url(&krate.name);
            crates_io = Some(url);
            // there's more cloning than necessary here but this is much cleaner than zero-copying!

            let CratesIoCrateResponse {
                description,
                repository,
                documentation,
            } = res.clone();

            if krate.repo.is_some() && krate.repo == repository {
                errors.push(format!("Please remove {}'s repo in ecosystem.json since \
                        it duplicates the value on crates.io", &krate.name));
            }

            if krate.description.is_some() && krate.description == description {
                errors.push(format!("Please remove {}'s description in ecosystem.json since \
                        it duplicates the value on crates.io", &krate.name));
            }

            if krate.docs.is_some() && krate.docs == documentation {
                errors.push(format!("Please remove {}'s docs in ecosystem.json since \
                        it duplicates the value on crates.io", &krate.name));
            }

            return CompiledCrate {
                crates_io,
                repo: krate.repo.clone().or(repository),
                description: krate.description.clone().or(description),
                docs: krate.docs.clone().or(documentation),
                tags: krate.tags.clone(),
            }
        }
        // the crate was not found on crates io
    }
    // we don't care if it's on crates io

    CompiledCrate {
        crates_io: None,
        repo: krate.repo.clone(),
        description: krate.description.clone(),
        docs: krate.docs.clone(),
        tags: krate.tags.clone(),
    }
}

fn parse_json_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, Box<Error>> {
    let f = File::open(path)?;
    Ok(serde_json::from_reader(f)?)
}

fn crates_io_url(crate_name: &str) -> String {
    format!("https://crates.io/api/v1/crates/{}", crate_name)
}
