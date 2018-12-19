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

#[derive(Serialize, Deserialize)]
struct AreWeGuiYetTemplateArgs {
    crates: Vec<Crate>,
    /// Collection of tags.
    ///
    /// Some tags may have descriptions.
    tags: HashMap<String, Option<String>>,
    newsfeed: Vec<NewsfeedEntry>,
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
    /// Attempt to load cache from path; creates empty cache on failure
    fn new(path: &str) -> Self {
        parse_json_file(path)
            .unwrap_or_else(|_| Default::default())
    }

    /// Get crate meta data from crates io API, and cache the result
    fn get_crates_io(&mut self, name: &str) -> Result<&Option<CrateResponse>, Box<Error>> {
        // We can perhaps make this better with NLL?
        if !self.crates_io.contains_key(name) {
            println!("Cache miss. Requesting data for {}", name);
            let url = crates_io_url(name);
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

    /// Uses data from crates.io if there is no custom repo specified for the crate.
    ///
    /// No fields will be overwritten if they are already specified.
    fn get_crate_info(&mut self, krate: &mut Crate) {
        if krate.repo.is_some() {
            return;
        }

        let res = self.get_crates_io(&krate.name)
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

            let mut cache = Cache::new(CACHE_FILE);

            if clean {
                cache.remove_cache(CACHE_FILE)
                    .expect(CACHE_FILE_DELETION_FAILED);
                println!("Cache file removed.");
            }

            // TODO: verify that every tag in the tags file is actually used
            // TODO: Verify that tag names and descriptions don't contain unwanted characters.
            // TODO: fill in auto-populated crate data from crates.io
            // right now it just calls old publish method, which has issues
            publish(&mut cache);

            if !verify_only {
                // TODO: emit HTML
            }
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
fn publish(cache: &mut Cache) {
    let mut crates: Vec<Crate> = parse_json_file(ECOSYSTEM)
        .expect("Failed to parse ecosystem.json");
    let mut tags: HashMap<String, Option<String>> = parse_json_file(ECOSYSTEM_TAGS)
        .expect("Failed to parse ecosystem_tags.json");
    let newsfeed: Vec<NewsfeedEntry> = parse_json_file(NEWSFEED)
        .expect("Failed to parse newsfeed.json");

    // TODO: Lack of non-lexical lifetimes means we need a special scope for used_tags
    // (used_tags borrows crates)
    {
        // figure out which tags we used and generate missing crate information
        let mut used_tags = HashSet::new();

        println!("Found {} crates.", crates.len());

        for krate in &mut crates {
            cache.get_crate_info(krate);
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

    compile_templates_and_write(
        &awgy,
        TEMPLATE_SOURCE_GLOB,
        INDEX_HTML_TEMPLATE_NAME,
        INDEX_HTML_OUTPUT_PATH,
    );
    println!("Site generated.");
}

fn parse_json_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, Box<Error>> {
    let f = File::open(path)?;
    Ok(serde_json::from_reader(f)?)
}

fn crates_io_url(crate_name: &str) -> String {
    format!("https://crates.io/api/v1/crates/{}", crate_name)
}

/// Compiles the tera templates from a hard coded path (the site directory).
fn compile_templates_and_write<P: AsRef<Path>>(
    awgy: &AreWeGuiYetTemplateArgs,
    template_source_glob: &str,
    index_html_template_name: &str,
    index_html_output_path: P,
) {
    let tera = compile_templates!(template_source_glob);
    // Render the template and remove newlines so people don't accidentally edit the compiled HTML
    // (we could actually minify it too)
    let index = tera.render(index_html_template_name, awgy)
        .expect("Failed to render templates")
        .replace("\r\n", " ")
        .replace("\n", " ");

    let mut out = File::create(index_html_output_path)
        .expect("Failed to create output file");
    out.write_all(index.as_bytes())
        .expect("Failed to write everything to the output file");
}