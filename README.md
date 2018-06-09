# Readme

## What is this?

Companion website to
[arewegameyet](http://arewegameyet.com),
[arewewebyet](http://www.arewewebyet.org), and
[arewelearningyet](http://www.arewelearningyet.com).

## Contributing

The "CLI" is not really a CLI yet but it *will* fetch missing for crates that don't specify
their repo in `ecosystem.json`.

If you know of a GUI crate, you can either submit an issue or make a PR!

Current format for `ecosystem.json` is

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
and the CLI will generate the `base.tera.html`.

`ecosystem_tags.json` lists descriptions for tags. There should not be any unused tags listed
in there and not all tags need to have a description, so not all tags need to be in that file.

n.b., Because the CLI uses hyper which brings in tokio and futures, it takes awhile to compile
and requires about half a gigabyte of disk space.

## TODO

Major undertakings remaining:
 - Crate search based on the tags and an optional query
 - Decide how we want to record approaches
 - Build the page for the newsfeed and start gathering posts
 - A reviewer or reviewers for blog posts and approaches written exclusively for our repo
 - Add commands to the CLI so people never have to touch the JSON files
 - Pull missing data from github and possibly other sites (bitbucket/gitlab?) if they have
  nice APIs!
 - Handle tag overflow in the crate cards
 - Do we want to support listing Pros/Cons in addition to crate descriptions?
 - Live badges for crate cards

And less major:
 - Move tag descriptions from title attribute into hover tooltip.
