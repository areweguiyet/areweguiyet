# Are We GUI Yet?

[![CI](https://github.com/areweguiyet/areweguiyet/actions/workflows/ci.yml/badge.svg)](https://github.com/areweguiyet/areweguiyet/actions/workflows/ci.yml)
[![GitHub Pages Deployment](https://github.com/areweguiyet/areweguiyet/actions/workflows/pages/pages-build-deployment/badge.svg)](https://github.com/areweguiyet/areweguiyet/actions/workflows/pages/pages-build-deployment)

Want to find crates for GUI in Rust? Then you've come to the right place!

This is a companion website to [arewegameyet](http://arewegameyet.com),
[arewewebyet](http://www.arewewebyet.org), and
[arewelearningyet](http://www.arewelearningyet.com).


## Status

The site is maintained as best as possible. If you know of a new blog post or
crate related to GUI development in Rust, please see [CONTRIBUTING.md] for how
to add it.

Data from external sources are updated weekly.

[CONTRIBUTING.md]: https://github.com/areweguiyet/areweguiyet/blob/master/CONTRIBUTIN.md


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
