# Readme

Want to find crates for GUI in Rust? Then you've come to the right place!

## Status

This site is maintained as best as possible. The next steps likely involve
replacing it with an automated system more like [lib.rs](https://lib.rs).

CI runs weekly to update crate information from crates.io, if it is not
overriden by the crate's configuration in this repo.

## What is this?

Companion website to
[arewegameyet](http://arewegameyet.com),
[arewewebyet](http://www.arewewebyet.org), and
[arewelearningyet](http://www.arewelearningyet.com).

## Contributing

To make it easy for people (hey, that's you! 😊) to contribute, AreWeGuiYet uses a custom
Rust CLI. It's currently a work in progress and is a little rough around the edges.

The workflow:

 * fork AreWeGuiYet
 * clone your fork
 * `cd` to the CLI directory (`cli`) (it currently uses relative paths 😬)
 * Build and run the CLI!
    * Usage: `cargo run -- [--clean] <command> [flags]`
    * Help: `cargo run -- help`
 * When you're done, commit your changes, push to your fork, and...
 * [Open a pull request!](https://github.com/areweguiyet/areweguiyet/compare)

Building the CLI requires Rust 1.31+. The CLI maintains a cache so it doesn't hammer any APIs
with more calls than it needs to. If you downloaded the repo awhile ago, please be sure to run
with the `--clean` flag.

If you need assistance (or especially if there's a bug, of which there are certainly many),
please open an issue (it's much appreciated! 💖).

**Please note:** If you find yourself editing any `.json` files because the CLI doesn't work or
you think it's dumb (a reasonable opinion), it would be appreciated to add to the *top* of the
file when adding new entries.

**n.b.**, Because the CLI uses reqwest, which brings in hyper and tokio, it may take awhile to
compile and requires almost a gigabyte of disk space.

### To submit a crate

Run the `framework` command from the `cli` directory like so:

```
cargo run -- framework
```

The CLI will walk you through the rest and will help make sure that AreWeGuiYet doesn't become
stale and contains the most recent information. If you make any errors, you can either `ctrl-c`
to exit and start over **or** carry on: When the CLI is done, you will find the `ecosystem.json`
populated with your additions. You can edit in any necessary changes. The main purpose of the CLI
is to encourage using defaults (which are written and updated by crate maintainers) and follow a
few consistency guidelines.

If you use the defaults, the CLI will automatically pull the most recent info from crates.io
whenever `publish` is run.

Now you're ready to open your PR!

### To submit a news link or post

This part of the CLI is unfinished as of writing. However, you may submit links and posts by
editing the `newsfeed.json` file directly, for now. The CLI `publish` command will update the HTML
appropriately (or yell at you if you made any oopsie woopsies; just like borrowchk!).

### Tags

`ecosystem_tags.json` lists descriptions for tags. There should not be any unused tags listed
in there and not all tags need to have a description, so not all tags need to be in that file.

## Organization

The `docs` directory is the build directory for the HTML content. `docs` is committed to master
and is used as the web root for GitHub pages. Non-HTML files in the `docs` directory are anything
that the HTML pages depend on. Some of these are not generated and are hand written.

The `site` directory contains the Tera template files, which are used by the CLI to generate the
HTML in the `docs` directory.

The JS code on the main page is a little sloppy. If there are compatibility issues with your
browser please open an issue!

The `cli` directory contains the CLI tool which is used to create new entries on the website and
generate the HTML.

## TODO

Major undertakings remaining:
 - Crate search based on the tags and an optional query
 - tag commands
 - tag normalization
 - markdown sanitization, better sanitization in `.raw.html` files
 - Pull missing data from github and possibly other sites (bitbucket/gitlab?) if they have
  nice APIs!
    - The original author has no intention of implementing this but if you want a challenge this
        could be a fun project!
 - Do we want to support
    - listing Pros/Cons in addition to crate descriptions?
    - Screenshots?
 - Live badges for crate cards

And less major:
 - Move tag descriptions from title attribute into hover tooltip.
