use newsfeed::*;

use serde::de::DeserializeOwned;

use clap::{App, SubCommand, Arg, AppSettings, ArgGroup};

use std::path::Path;
use std::io::Write;
use std::fs::File;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io;
use std::io::BufRead;

// source files
const NEWSFEED: &str = "../newsfeed.json";
const ECOSYSTEM: &str = "../ecosystem.json";
const COMPILED_ECOSYSTEM: &str = "../docs/compiled_ecosystem.json";
const ECOSYSTEM_TAGS: &str = "../ecosystem_tags.json";

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

// error messages
const READ_LINE_PANIC_MESSAGE: &str = "Failed to read line";

// TODO: There's plenty more messages encoded as string literals; easy PR! ;^)

/// Prints a message, and fills in a single placeholder with a default value
///
/// The placeholder is filled with nothing if default is None, otherwise it is filled
/// with " (default.unwrap())".
macro_rules! println_default {
    ($msg:expr, $default:expr) => {
        if let Some(default) = $default {
            println!($msg, format!(" ({})", default));
        } else {
            println!($msg, "");
        }
    }
}

/// The arguments needed for Tera to render the template.
#[derive(Serialize, Deserialize)]
struct AreWeGuiYetTemplateArgs {
    crates: Vec<Crate>,
    /// Collection of tags.
    ///
    /// Some tags may have descriptions.
    tags: HashMap<String, Option<String>>,
    news_posts: Vec<NewsfeedTemplateArgs>,
    news_links: Vec<NewsfeedTemplateArgs>,
    page_title: Option<String>,
}

/// Crate info in ecosystem.json
#[derive(Serialize, Deserialize)]
struct Crate {
    name: String,
    /// Should be either missing or true; implied to be false
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    skip_crates_io: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<String>,
    tags: Vec<String>,
}

/// The template args mas all variants to NewsFeedLinks for template simplicity
#[derive(Serialize, Deserialize)]
struct NewsfeedTemplateArgs {
    title: String,
    author: String,
    link: String,
    order: u32,
}

impl NewsfeedTemplateArgs {
    fn new(n: &NewsfeedEntry, link: &String) -> NewsfeedTemplateArgs {
        // I should get a prize for being this efficient!
        NewsfeedTemplateArgs {
            title: n.title.clone(),
            author: n.author.clone(),
            order: n.order.clone(),
            link: link.clone(),
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
            let url = crates_io_api_url(name);
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
        .arg(Arg::with_name("clean")
            .long("clean")
            .help("Force refreshes the cache, making new network requests"))
        .subcommand(SubCommand::with_name("publish")
            .about("Publishes generated HTML to docs directory. Fork the repo, push the resulting \
                  changes, and then open a PR on Github to share your changes!")
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

    let mut cache = Cache::new(CACHE_FILE);

    if matches.is_present("clean") {
        cache.remove_cache(CACHE_FILE)
            .expect(CACHE_FILE_DELETION_FAILED);
        println!("Cache file removed.");
    }

    match matches.subcommand() {
        ("publish", args) => {
            let verify_only = match args {
                Some(args) => args.is_present("verify-only"),
                None => false,
            };

            publish(&mut cache, verify_only);
        },
        ("framework", _) => {
            framework(&mut cache);
        },
        ("news", _) => {
            unimplemented!();
        },
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
        let compiled_crate = get_crate_info(krate, cache, &mut errors);
        compiled_ecosystem.insert(krate.name.clone(), compiled_crate);
    }

    // compile the templates
    let mut tera = compile_templates!(TEMPLATE_SOURCE_GLOB);
    tera.autoescape_on(vec![".tera.html"]);

    // compile news posts and gather links
    let mut news_post_rendered_html = HashMap::new();
    let mut news_posts = Vec::new();
    let mut news_links = Vec::new();

    for entry in &newsfeed {
        match &entry.source {
            NewsfeedSource::Link { link } => {
                news_links.push(NewsfeedTemplateArgs::new(&entry, link));
            },
            NewsfeedSource::Post { file_name } => {
                // open the file containing the markdown
                let mut markdown_path = NEWSFEED_POST_MARKDOWN_ROOT.to_string();
                markdown_path.push_str(file_name);
                let markdown = fs::read_to_string(&markdown_path)
                    .expect("Failed to read markdown post");
                // parse and render the markdown for this post
                let parser = pulldown_cmark::Parser::new(&markdown);
                let mut rendered_content = String::new();
                pulldown_cmark::html::push_html(&mut rendered_content, parser);
                // render to a template
                let post_content = PostTemplateArgs {
                    page_title: entry.title.clone(),
                    post_content: rendered_content,
                };
                let rendered_page = tera.render(NEWSFEED_POST_HTML_TEMPLATE_NAME, &post_content)
                    .expect("Failed to render hosted news post")
                    .replace("\r\n", " ")
                    .replace("\n", " ");
                // save the rendered template so we can output it later
                let mut link = file_name.replace(".md", ".html");
                news_post_rendered_html.insert(link.clone(), rendered_page);
                // record the news post so it can be rendered into other pages on the site
                // TODO: this is very fragile... (and arguably dangerous)
                link.insert_str(0, NEWSFEED_POST_HTML_LINK_ROOT);
                news_posts.push(NewsfeedTemplateArgs::new(&entry, &link));
            }
        }
    }

    println!("Found {} community news links.", news_links.len());
    println!("Found {} hosted news posts.", news_posts.len());

    let mut awgy = AreWeGuiYetTemplateArgs {
        crates,
        tags,
        news_posts,
        news_links,
        page_title: None,
    };

    // Render the templates and remove newlines so people don't accidentally edit the compiled HTML
    // (we could actually minify it too)
    awgy.page_title = None;
    let index = tera.render(INDEX_HTML_TEMPLATE_NAME, &awgy)
        .expect("Failed to render template")
        .replace("\r\n", " ")
        .replace("\n", " ");

    awgy.page_title = Some("News Feed".to_string());
    let newsfeed = tera.render(NEWSFEED_HTML_TEMPLATE_NAME, &awgy)
        .expect("Failed to render template")
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

        output_html(INDEX_HTML_OUTPUT_PATH, &index)
            .expect("Failed to create index page");
        output_html(NEWSFEED_HTML_OUTPUT_PATH, &newsfeed)
            .expect("Failed to create newsfeed page");
        
        // output the rendered markdown posts
        for (mut file_name, rendered_html) in news_post_rendered_html.into_iter() {
            file_name.insert_str(0, NEWSFEED_POST_HTML_OUTPUT_ROOT);
            output_html(&file_name, &rendered_html)
                .expect("Failed to create hosted post page");
        }

        println!("Site written to disk.");
    } else {
        println!("Skipping writing the site.");
    }
}

// this is a little sloppy but hey! it works...
fn output_html(path: &str, page: &str) -> io::Result<()> {
    let mut out_index = File::create(path)?;
    out_index.write_all(page.as_bytes())
}

fn framework(cache: &mut Cache) {
    let mut crates: Vec<Crate> = parse_json_file(ECOSYSTEM)
        .expect("Failed to parse ecosystem.json. This must be fixed before we can add more.");

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut krate = Crate {
        name: String::new(),
        skip_crates_io: false,
        docs: None,
        repo: None,
        description: None,
        tags: Vec::new(),
    };

    println!("\tADDING GUI FRAMEWORK");

    loop {
        println!("What is the name of the GUI framework?");
        get_input_non_empty(&mut handle, &mut buffer);
        krate.name.clear();
        krate.name.push_str(&buffer);

        // make sure this crate isn't already present
        let mut already_done = false;
        for c in &crates {
            let this_name = krate.name.to_lowercase();
            let other_name = c.name.to_lowercase();
            if this_name == other_name {
                println!("The crate {} is already on the website!", krate.name);
                already_done = true;
                break;
            }
        }
        if already_done {
            continue;
        }

        // make a request to crates io about this crate
        match cache.get_crates_io(&krate.name) {
            Ok(None) => {
                // this crate is not on crates io

                println!("This crate does not appear on crates.io. Do you want to continue \
                    without linking to a crate on crates.io? (y/n)\
                    \n\n(Please check your spelling and enter no if there is a typo; \
                    crate names must match exactly.)");
                if get_input_yes(&mut handle, &mut buffer) {
                    break;
                } else {
                    continue;
                }
            }
            Ok(Some(res)) => {
                // this crate is on crates io

                println!("There is a crate on crates.io with this name! Woo!\n");
                println!("  Crate: {}", &krate.name);
                println!("  Link: {}", crates_io_url(&krate.name));
                println!("  Repo: {:?}", &res.repository);
                if let Some(desc) = &res.description {
                    let desc = desc.trim();
                    if desc.len() > 70 {
                        println!("  Description: {}...", &desc[..70]);
                    } else {
                        println!("  Description: {}", desc);
                    }
                }
                println!("\nIs this the correct crate? (y/n)");
                if get_input_yes(&mut handle, &mut buffer) {
                    // this is the correct crate, set some defaults pulled from crates.io
                    krate.skip_crates_io = false;
                    // the description is the most important one to trim but we might as well
                    // trim all
                    krate.description = res.description.as_ref()
                        .map(|s| s.trim().to_owned());
                    krate.repo = res.repository.as_ref()
                        .map(|s| s.trim().to_owned());
                    krate.docs = res.documentation.as_ref()
                        .map(|s| s.trim().to_owned());
                    break;
                } else {
                    // not the correct crate
                    println!("Please check if you made any spelling mistakes in the crate \
                        name.\n");
                    println!("Do you want to continue without linking to a crate on \
                        crates.io? (if no, you will have to enter a different crate name) \
                        (y/n)");
                    if get_input_yes(&mut handle, &mut buffer) {
                        krate.skip_crates_io = true;
                        break;
                    } else {
                        continue;
                    }
                }
            }
            Err(e) => {
                // there was a problem fetching
                eprintln!("Error fetching from crates.io: {}", e);
                println!("Failed to fetch from crates.io. Try again? (y/n)");
                if get_input_yes(&mut handle, &mut buffer) {
                    break;
                } else {
                    continue;
                }
            }
        };
    }

    println!("Press enter to accept the defaults provided in `()`, if any.\n");

    // this gets a little sloppy, and could probably be done better

    // we abuse the Crate instance here; if it already has stuff in it, assume it is a default
    // if we want to use the defaults (which are pulled from crates io)
    //   - the user enters nothing or whitespace
    //   - we erase the default (that we had just stored in the Crate instance) with None

    println_default!("Description{}:", krate.description.as_ref());
    if get_input_allow_empty(&mut handle, &mut buffer) == None {
        krate.description = None
    } else {
        krate.description = Some(buffer.clone());
    }

    println_default!("Docs{}:", krate.docs.as_ref());
    if get_input_allow_empty(&mut handle, &mut buffer) == None {
        krate.docs = None
    } else {
        krate.docs = Some(buffer.clone());
    }

    println_default!("Repo{}:", krate.repo.as_ref());
    if get_input_allow_empty(&mut handle, &mut buffer) == None {
        krate.repo = None
    } else {
        krate.repo = Some(buffer.clone());
    }

    // tags
    loop {
        println!("Enter the name of a tag (enter nothing to finish):");
        get_input_allow_empty(&mut handle, &mut buffer);
        if buffer.len() == 0 {
            break;
        }
        krate.tags.push(buffer.to_string());
    }

    // add this crate to the front of the list
    // JSON doesn't like trailing commas so adding to the front of the list makes diffs nicer
    // (only the lines actually added appear changed)
    crates.insert(0, krate);

    let out = File::create(ECOSYSTEM)
        .expect("Failed to create/open ecosystem.json.");
    serde_json::to_writer_pretty(out, &crates)
        .expect("Failed to write the updated ecosystem.json.");

    println!("Updated ../ecosystem.json. Review your changes and make any edits there.\n");
    println!("When you are done, please run `cargo run -- publish` to generate HTML.");
}

/// Clear buffer and fill it with the next line of (trimmed) input
fn get_input_non_empty(handle: &mut io::StdinLock, buffer: &mut String) {
    loop {
        buffer.clear();
        handle.read_line(buffer)
            .expect(READ_LINE_PANIC_MESSAGE);
        // TODO: Do without allocating?
        *buffer = buffer.trim().to_string();
        if buffer.len() != 0 {
            return;
        }
        println!(" (you must enter a value)");
    }
}

/// Clear buffer and fill it with the next line of (trimmed) input
///
/// Returns None when the input is empty
fn get_input_allow_empty(handle: &mut io::StdinLock, buffer: &mut String) -> Option<()> {
    buffer.clear();
    handle.read_line(buffer)
        .expect(READ_LINE_PANIC_MESSAGE);
    // TODO: Do without allocating?
    *buffer = buffer.trim().to_string();
    if buffer.len() == 0 {
        None
    } else {
        Some(())
    }
}

fn get_input_yes(handle: &mut io::StdinLock, buffer: &mut String) -> bool {
    get_input_non_empty(handle, buffer);
    buffer.starts_with("y") || buffer.starts_with("Y")
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

fn crates_io_api_url(crate_name: &str) -> String {
    format!("https://crates.io/api/v1/crates/{}", crate_name)
}

fn crates_io_url(crate_name: &str) -> String {
    format!("https://crates.io/crates/{}", crate_name)
}
