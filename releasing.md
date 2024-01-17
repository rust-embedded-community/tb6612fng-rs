# How To Release The Crate
This guide is only relevant for maintainers.

This crate is set up to be released using [`cargo-release`](https://crates.io/crates/cargo-release),
please install it and use it to do the release. It also takes care of updating the [changelog](CHANGELOG.md).

To do the release:
1. Make sure that your local clone of the repository is up-to-date
2. Switch to a new feature branch for the release
3. Run `cargo release [level]` (see their documentation for more details) to dry-run it.
   Make sure to choose the appropriate level based on semantic versioning!
4. Once you're happy with the result append `--execute`
5. After the release push the created tag & commit, create a PR & merge it
6. Create a release on GitHub based on the tag
