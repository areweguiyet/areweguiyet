use clap::{App, SubCommand, Arg, AppSettings, ArgGroup};

pub fn execute_cli() {
    let matches = App::new("Areweguiyet CLI")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("CLI for extending areweguiyet website")
        .subcommand(SubCommand::with_name("publish")
            .about("Publishes generated HTML to docs directory. Fork the repo, push the resulting \
                  changes, and then open a PR on Github to share your changes!"))
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
}