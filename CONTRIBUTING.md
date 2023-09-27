# Contributing

Thank you for your interest in contributing!

If you want to submit a crate or a news link, please follow the instructions
below.


## Submitting a crate

Add the crate to the relevant category under the `ecosystem` directory, and
open a pull request.

The allowed keys are as follows:

```toml
# A crate on crates.io.
[extra.crates.my-crate]
# The name of the crate. Defaults to the crate name, but may be capitalized
# or named differently if desired.
name = "MyCrate"
# A description of the crate. Defaults to the description on crates.io.
description = "A crate to do stuff"
# The graphics / layout technology that the crate uses.
#
# This is required for the `bindings`, `framework` and `toolkit` categories.
technology = "native" | "custom" | "web"
# The platforms that this crate supports.
platform = ["desktop", "mobile", "web"]
# Location of the crate's code repository.
#
# Defaults to what is specified on crates.io.
repo = "https://domain.invalid/mycrate/git"
# Location of the crate's documentation.
#
# Defaults to what is specified on crates.io, or otherwise to docs.rs.
docs = "https://domain.invalid/mycrate/docs"
```

For crates on crates.io, you should usually only include `name` and `tags`,
and leave all other fields blank. The remaining fields will be populated with
the crate's own defaults, which are written and updated by the crate
maintainers themselves. AWGY is rebuilt weekly to keep these fields (e.g.
`docs`, `repo`) up to date.

Your pull request will fail to build if `ecosystem.toml` includes these fields
with the same data as crates.io.

For projects/libraries/etc not on crates.io, or that are not well represented
by a single crate, you can provide all the fields, along with `skip-crates-io`
to prevent checking against crates.io.


## Submitting a news link

Add the post to the `content/newsfeed/links.toml` file using the keys
documented therein, and open a pull request.


## Changing the HTML/CSS

The site uses a Rust CLI script to fetch data from various sources such as
crates.io. To use it, run `cargo run -- fetch` every time the `ecosystem.toml`
file has been updated. The CLI maintains a cache so it doesn't hammer any APIs
with more calls than it needs to. If you fear that the data might be out of
date, you can run `cargo run -- clean` to refresh the state.

Once you have fetched the external data, you can use the static site generator
[Zola] to serve the site locally and test any changes. See their website for
more usage instructions.

[Zola]: https://www.getzola.org/
