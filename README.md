# Readme

## What is this?

Companion website to
[arewegameyet](http://arewegameyet.com),
[arewewebyet](http://www.arewewebyet.org), and
[arewelearningyet](http://www.arewelearningyet.com).

## Contributing

The "CLI" is not really a CLI yet but it *will* fetch missing for crates that don't specify
their repo in `ecosystem.json`.

If you know of a GUI crate or want to update the news feed, you can either submit an issue or
make a PR!

To add a **crate**, add it to `ecosystem.json`, Current format for `ecosystem.json` is

```js
{
    name: String,
    description: Option<String>,
    docs: Option<String>,
    // leave this repo blank if you want to pull information from crates.io
    repo: Option<String>,
    // specify a list of a tags that are relevant for searching through GUI crates
    tags: Vec<String>,
}
```

Once you add a crate to the JSON file, you can simply do `cargo run` from the `cli` directory
and the CLI will generate the website (which is outputted into the `docs` directory).

The process is the same for adding to the **newsfeed**, except you edit `newsfeed.json`. The
format for each entry is

```rust
{
    kind: "Link",
    title: String,
    author: String,
    link: String,
}
```

**Please add to the top of the file.** Only links are supported at the moment, but if you have
a suggestion for another format, or would like to write a blog post exclusively for this site,
open an issue!

`ecosystem_tags.json` lists descriptions for tags. There should not be any unused tags listed
in there and not all tags need to have a description, so not all tags need to be in that file.

n.b., Because the CLI uses reqwest, which brings in hyper and tokio, it may take awhile to
compile and requires about half a gigabyte of disk space.

## Organization

The `docs` directory is the build directory for the HTML content. `docs` is committed to master and
is used as the web root for GitHub pages. Non-HTML files in the `docs` directory are anything that 
the HTML pages depend on, including `ecosystem.json` and support CSS and JS files.

The `site` directory contains the Tera template files, which are used by the CLI to generate the 
HTML in the `docs` directory.

The JS code uses let statements and the fetch API, but otherwise is very vanilla and old school.
To keep things simple (the JS on the page should be minimal anyways), there are no dependencies.
If there are compatibility issues with your browser please open an issue!

The `cli` directory contains a CLI tool which is used to create new entries on the website and
generate the HTML.

## TODO

Major undertakings remaining:
 - Crate search based on the tags and an optional query
 - Decide how we want to record approaches
 - Refactor to support the dedicated newsfeed page (for when we have more than 3 posts)
 - A reviewer or reviewers for blog posts and approaches written exclusively for our repo
 - Add commands to the CLI so people never have to touch the JSON files
 - Pull missing data from github and possibly other sites (bitbucket/gitlab?) if they have
  nice APIs!
 - Handle tag overflow in the crate cards
 - Do we want to support listing Pros/Cons in addition to crate descriptions?
 - Live badges for crate cards

And less major:
 - Move tag descriptions from title attribute into hover tooltip.
