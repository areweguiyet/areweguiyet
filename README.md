# Are We GUI Yet?

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


### Submitting a crate

Add the crate to the `ecosystem.toml` file (using the keys documented
therein), and open a pull request.

You are strongly encouraged to default to the crate's own defaults, which are
written and updated by the crate maintainers themselves.


### Submitting a news link

Add the post to the `newsfeed.toml` file, and open a pull request.


### Using the CLI

AreWeGuiYet uses a custom Rust CLI to fetch data from crates.io. It's
currently a work in progress and is a little rough around the edges.

Building the CLI requires Rust 1.31+. The CLI maintains a cache so it doesn't
hammer any APIs with more calls than it needs to. If you downloaded the repo
awhile ago, please be sure to run with the `--clean` flag.

If you need assistance (or especially if there's a bug, of which there are
certainly many), please open an issue (it's much appreciated! 💖).


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
