use crate::newsfeed::*;

use serde::de::DeserializeOwned;

use clap::{Arg, ArgAction, ArgGroup, Command};
use reqwest::blocking::Client as HttpClient;

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::hash::BuildHasher;
use std::io;
use std::io::Write;
use std::path::Path;

// source files
const NEWSFEED: &str = "../newsfeed.json";
const ECOSYSTEM: &str = "../ecosystem.toml";
const COMPILED_ECOSYSTEM: &str = "../docs/compiled_ecosystem.json";
const ECOSYSTEM_TAGS: &str = "../ecosystem_tags.toml";

// templates
const TEMPLATE_SOURCE_GLOB: &str = "../site/**/*.tera*.html";

const INDEX_HTML_TEMPLATE_NAME: &str = "index.tera.html";
const INDEX_HTML_OUTPUT_PATH: &str = "../docs/index.html";

const NEWSFEED_HTML_TEMPLATE_NAME: &str = "newsfeed.tera.html";
const NEWSFEED_HTML_OUTPUT_PATH: &str = "../docs/newsfeed/index.html";

const NEWSFEED_POST_HTML_OUTPUT_ROOT: &str = "../docs/newsfeed/";
const NEWSFEED_POST_HTML_LINK_ROOT: &str = "/newsfeed/";
const NEWSFEED_POST_MARKDOWN_ROOT: &str = "../localposts/";
const NEWSFEED_POST_HTML_TEMPLATE_NAME: &str = "newsfeed_post.tera.raw.html";

// cache
const CACHE_FILE: &str = "../cache.json";
const CACHE_FILE_DELETION_FAILED: &str = "Failed to remove the cache file. Try deleting it \
    manually and running without the clean option.";
const CACHE_CLIENT_USER_AGENT: &str = "areweguiyet_cli (areweguiyet.com)";

// TODO: There's plenty more messages encoded as string literals; easy PR! ;^)

/// Info in ecosystem.toml
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Ecosystem {
    #[serde(rename = "crate")]
    crates: HashMap<String, Crate>,
}

/// Crate info in ecosystem.toml
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Crate {
    name: Option<String>,
    /// Should be either missing or true; implied to be false
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    #[serde(rename = "skip-crates-io")]
    skip_crates_io: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

/// The arguments needed for Tera to render the template.
#[derive(Serialize, Deserialize, Debug)]
struct AreWeGuiYetTemplateArgs {
    /// Collection of tags.
    ///
    /// Some tags may have descriptions.
    tags: HashMap<String, Option<String>>,
    news_posts: Vec<NewsfeedTemplateArgs>,
    news_links: Vec<NewsfeedTemplateArgs>,
    page_title: Option<String>,
}

/// The template args mas all variants to NewsFeedLinks for template simplicity
#[derive(Serialize, Deserialize, Debug)]
struct NewsfeedTemplateArgs {
    title: String,
    author: String,
    link: String,
    order: u32,
}

impl NewsfeedTemplateArgs {
    fn new(n: &NewsfeedEntry, link: &str) -> NewsfeedTemplateArgs {
        // I should get a prize for being this efficient!
        NewsfeedTemplateArgs {
            title: n.title.clone(),
            author: n.author.clone(),
            order: n.order,
            link: link.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PostTemplateArgs {
    page_title: String,
    post_content: String,
}

fn is_false(b: &bool) -> bool {
    !*b
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
#[derive(Serialize, Deserialize, Default)]
struct Cache {
    crates_io: HashMap<String, Option<CratesIoCrateResponse>>,
    #[serde(skip)]
    client: Option<HttpClient>,
}

impl Cache {
    /// Attempt to load cache from path; creates empty cache on failure
    fn new(path: &str) -> Self {
        parse_json_file(path).unwrap_or_else(|_| Default::default())
    }

    fn client(&mut self) -> &mut HttpClient {
        if self.client.is_none() {
            let mut builder = HttpClient::builder();
            builder = builder.user_agent(CACHE_CLIENT_USER_AGENT);
            let client = builder
                .build()
                .expect("Reqwest client build error (TLS backend init failure)");
            self.client = Some(client);
        }

        self.client.as_mut().unwrap()
    }

    /// Get crate meta data from crates io API, and cache the result
    fn get_crates_io(
        &mut self,
        name: &str,
    ) -> Result<&Option<CratesIoCrateResponse>, Box<dyn Error>> {
        // We can perhaps make this better with NLL?
        if !self.crates_io.contains_key(name) {
            println!("Cache miss. Requesting data for {}", name);
            let url = crates_io_api_url(name);
            let req = self.client().get(&url);
            let res = req.send()?;
            let parsed: Option<CratesIoEnvelopeResponse> = match res.status() {
                reqwest::StatusCode::OK => res.json()?,
                reqwest::StatusCode::NOT_FOUND => None,
                status => {
                    return Err(format!(
                        "Unexpected request status ({}) while fetching {}",
                        status, &url,
                    )
                    .into());
                }
            };
            self.crates_io
                .insert(name.to_string(), parsed.map(|x| x.krate));
        }
        Ok(&self.crates_io[name])
    }

    /// Save the cache to disk at path
    fn write_cache(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let out = File::create(path)?;

        // This is fatal because running the app again will cause it fail as the cache file exists
        // but may contain garbage/be empty
        serde_json::to_writer_pretty(out, self).expect("Failed to write the cache file.");

        Ok(())
    }

    /// Removes the cache file and clears the cache data
    fn remove_cache(&mut self, path: &str) -> Result<(), io::Error> {
        self.crates_io.clear();
        // remove the cache (NotFound errors are fine)
        fs::remove_file(path).or_else(|err| {
            if err.kind() != io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(err)
            }
        })
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

/// A hasher that should be deterministic accross all runs.
///
/// AWGY is hosted on Github pages, so our rendered/generated content is part of the repo's history.
///
/// The CLI caches some information on publish to the `compiled_ecosystem.json`, and this JSON
/// involes a map object. The resulting order of the JSON is dependent on the iteration order of the
/// hashmap used to generate the JSON.
///
/// Together, this means every time the CLI is run, the compiled JSON is changed if the iteration
/// order of the hashmap used to generate it changes. By default, `HashMap`s use the `RandomState`
/// hasher builder, so the iteration order is different every CLI invocation. This creates some
/// noise in the diffs, and it can make it challenging to verify contributors have published their
/// latest changes.
///
/// `ConstantState` is an attempt to make this deterministic across all CLI invocations. This is an
/// "attempt" because, Rust does not technically guarantee what "DefaultHasher::new()" uses as its
/// hash algorithm or what state it uses, so in theory this could change between Rust versions.
///
/// This is of course unlikely, so this should 80% solve the issue.
struct ConstantState;

impl BuildHasher for ConstantState {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher::new()
    }
}

fn cli() -> Command {
    Command::new("Areweguiyet CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .about("CLI for extending areweguiyet website")
        .arg(Arg::new("clean")
            .long("clean")
            .help("Force refreshes the cache, making new network requests")
            .action(ArgAction::SetTrue))
        .subcommand(Command::new("publish")
            .about("Publishes generated HTML to docs directory. Fork the repo, push the resulting \
                  changes, and then open a PR on Github to share your changes!")
            .arg(Arg::new("verify-only")
                .long("verify-only")
                .help("Run all normal checks before publishing without generating HTML.")
                .action(ArgAction::SetTrue)))
        .subcommand(Command::new("news")
            .about("Adds a new news post from either a link or a markdown file.")
            .arg(Arg::new("link")
                .long("link")
                .short('l')
                .help("Adds a new news post from a link to another website"))
            .arg(Arg::new("post")
                .long("post")
                .short('p')
                .help("Creates a new news post hosted on Areweguiyet"))
            .group(ArgGroup::new("newsfeed_type")
                .args(["post", "link"])
                .required(true)))
}

pub fn execute_cli() {
    let matches = cli().get_matches();

    let mut cache = Cache::new(CACHE_FILE);

    if matches.get_flag("clean") {
        cache
            .remove_cache(CACHE_FILE)
            .expect(CACHE_FILE_DELETION_FAILED);
        println!("Cache file removed.");
    }

    match matches.subcommand() {
        Some(("publish", args)) => {
            let verify_only = args.get_flag("verify-only");

            publish(&mut cache, verify_only);
        }
        Some(("news", _)) => {
            unimplemented!();
        }
        _ => unreachable!(),
    }

    // update the cache
    match cache.write_cache(CACHE_FILE) {
        Ok(_) => println!("Cache updated."),
        Err(_) => println!("Nonfatal: Failed to write the cache"),
    }
}

/// Compile ecosystem info, cache result, and generate warnings
fn publish(cache: &mut Cache, verify_only: bool) {
    // Load all the information we need
    let ecosystem: Ecosystem = parse_toml_file(ECOSYSTEM).expect("failed to parse ecosystem.toml");
    let mut tags: HashMap<String, Option<String>> =
        parse_toml_file(ECOSYSTEM_TAGS).expect("failed to parse ecosystem_tags.toml");
    let newsfeed: Vec<NewsfeedEntry> =
        parse_json_file(NEWSFEED).expect("Failed to parse newsfeed.json");

    println!("Found {} crates.", ecosystem.crates.len());

    let mut errors = Vec::new();

    // TODO: Verify that tag names and descriptions don't contain unwanted characters.

    // verify that every tag in the tags file is actually used
    let mut used_tags = HashSet::new();
    for krate in ecosystem.crates.values() {
        used_tags.extend(krate.tags.iter())
    }
    // issue a warning if there are unsused tags in ecosystem_tags.toml
    for k in tags.keys() {
        if !used_tags.contains(k) {
            errors.push(format!("Tag \"{}\" is not used to describe any crate", k));
        }
    }
    // merge description-less used tags into ecosystem_tags
    for k in used_tags {
        tags.entry(k.to_string()).or_insert(None);
    }

    // merge missing crate information from crates io
    let mut compiled_ecosystem = HashMap::with_hasher(ConstantState);
    for (crate_id, krate) in &ecosystem.crates {
        let compiled_crate = get_crate_info(crate_id, krate, cache, &mut errors);
        compiled_ecosystem.insert(
            krate.name.clone().unwrap_or_else(|| crate_id.clone()),
            compiled_crate,
        );
    }

    // compile the templates
    let mut tera = tera::Tera::new(TEMPLATE_SOURCE_GLOB).expect("failed to parse templates");
    tera.autoescape_on(vec![".tera.html"]);

    // compile news posts and gather links
    let mut news_post_rendered_html = HashMap::new();
    let mut news_posts = Vec::new();
    let mut news_links = Vec::new();

    for entry in &newsfeed {
        match &entry.source {
            NewsfeedSource::Link { link } => {
                news_links.push(NewsfeedTemplateArgs::new(entry, link));
            }
            NewsfeedSource::Post { file_name } => {
                // open the file containing the markdown
                let mut markdown_path = NEWSFEED_POST_MARKDOWN_ROOT.to_string();
                markdown_path.push_str(file_name);
                let markdown =
                    fs::read_to_string(&markdown_path).expect("Failed to read markdown post");
                // parse and render the markdown for this post
                let parser = pulldown_cmark::Parser::new(&markdown);
                let mut rendered_content = String::new();
                pulldown_cmark::html::push_html(&mut rendered_content, parser);
                // render to a template
                let post_content = PostTemplateArgs {
                    page_title: entry.title.clone(),
                    post_content: rendered_content,
                };
                let context = tera::Context::from_serialize(post_content).unwrap();
                let rendered_page = tera
                    .render(NEWSFEED_POST_HTML_TEMPLATE_NAME, &context)
                    .expect("Failed to render hosted news post");
                // save the rendered template so we can output it later
                let mut link = file_name.replace(".md", ".html");
                news_post_rendered_html.insert(link.clone(), rendered_page);
                // record the news post so it can be rendered into other pages on the site
                // TODO: this is very fragile... (and arguably dangerous)
                link.insert_str(0, NEWSFEED_POST_HTML_LINK_ROOT);
                news_posts.push(NewsfeedTemplateArgs::new(entry, &link));
            }
        }
    }

    println!("Found {} community news links.", news_links.len());
    println!("Found {} hosted news posts.", news_posts.len());

    let mut awgy = AreWeGuiYetTemplateArgs {
        tags,
        news_posts,
        news_links,
        page_title: None,
    };

    // Render the templates and remove newlines so people don't accidentally edit the compiled HTML
    // (we could actually minify it too)
    awgy.page_title = None;
    let context = tera::Context::from_serialize(&awgy).unwrap();
    let index = tera
        .render(INDEX_HTML_TEMPLATE_NAME, &context)
        .expect("Failed to render template");

    awgy.page_title = Some("News Feed".to_string());
    let context = tera::Context::from_serialize(&awgy).unwrap();
    let newsfeed = tera
        .render(NEWSFEED_HTML_TEMPLATE_NAME, &context)
        .expect("Failed to render template");

    println!("Successfully rendered templates.");

    if !errors.is_empty() {
        eprintln!("The following issues are preventing HTML generation:");
        for i in &errors {
            println!("\t{}", i);
        }
        eprintln!("Failed to generate site.");
        return;
    }

    if !verify_only {
        fs::create_dir_all(Path::new(COMPILED_ECOSYSTEM).parent().unwrap())
            .expect("Failed to create directory");

        let mut out_compiled_ecosystem =
            File::create(COMPILED_ECOSYSTEM).expect("Failed to create compiled ecosystem file");
        serde_json::to_writer_pretty(&mut out_compiled_ecosystem, &compiled_ecosystem)
            .expect("Failed to write the compiled ecosystem to the output file");

        output_html(INDEX_HTML_OUTPUT_PATH, &index).expect("Failed to create index page");
        output_html(NEWSFEED_HTML_OUTPUT_PATH, &newsfeed).expect("Failed to create newsfeed page");

        // output the rendered markdown posts
        for (mut file_name, rendered_html) in news_post_rendered_html.into_iter() {
            file_name.insert_str(0, NEWSFEED_POST_HTML_OUTPUT_ROOT);
            output_html(&file_name, &rendered_html).expect("Failed to create hosted post page");
        }

        println!("Site written to disk.");
    } else {
        println!("Skipping writing the site.");
    }
}

fn output_html<P: AsRef<Path>>(path: P, page: &str) -> io::Result<()> {
    fs::create_dir_all(path.as_ref().parent().unwrap())
        .expect("Failed to create parent dir for HTML file");
    let mut out_index = File::create(path)?;
    out_index.write_all(page.as_bytes())
}

/// Merge saved data with data from crates io (if the crate is on crates io).
///
/// No fields will be overwritten if they are already specified.
///
/// Issues errors if the data from crates io is the same as the local data.
fn get_crate_info(
    crate_id: &str,
    krate: &Crate,
    cache: &mut Cache,
    errors: &mut Vec<String>,
) -> CompiledCrate {
    let crates_io;

    if !krate.skip_crates_io {
        let res = cache
            .get_crates_io(crate_id)
            .expect("Failed to fetch from Crates.io");

        if let Some(res) = res {
            let url = crates_io_url(crate_id);
            crates_io = Some(url);
            // there's more cloning than necessary here but this is much cleaner than zero-copying!

            let CratesIoCrateResponse {
                description,
                repository,
                documentation,
            } = res.clone();

            if krate.repo.is_some() && krate.repo == repository {
                errors.push(format!(
                    "Please remove {crate_id}'s repo in ecosystem.toml since \
                        it duplicates the value on crates.io",
                ));
            }

            if krate.description.is_some() && krate.description == description {
                errors.push(format!(
                    "Please remove {crate_id}'s description in ecosystem.toml since \
                        it duplicates the value on crates.io",
                ));
            }

            if krate.docs.is_some() && krate.docs == documentation {
                errors.push(format!(
                    "Please remove {crate_id}'s docs in ecosystem.toml since \
                        it duplicates the value on crates.io",
                ));
            }

            return CompiledCrate {
                crates_io,
                repo: krate.repo.clone().or(repository),
                description: krate.description.clone().or(description),
                docs: krate.docs.clone().or(documentation),
                tags: krate.tags.clone(),
            };
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

fn parse_json_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, Box<dyn Error>> {
    let f = File::open(path)?;
    Ok(serde_json::from_reader(f)?)
}

fn parse_toml_file<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T, Box<dyn Error>> {
    let s = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&s)?)
}

fn crates_io_api_url(crate_name: &str) -> String {
    format!("https://crates.io/api/v1/crates/{}", crate_name)
}

fn crates_io_url(crate_name: &str) -> String {
    format!("https://crates.io/crates/{}", crate_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        cli().debug_assert();
    }
}
