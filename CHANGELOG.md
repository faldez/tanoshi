# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- [tanoshi] check for update every 24 hours and send notification to admins if found
- [tanoshi-lib] Version struct now on tanoshi-lib, `verion` field in `Source` now is Version struct
- [tanoshi-lib] add lib_version to `Source` to identify `tanoshi-lib` version is used

### Changed
- [tanoshi] optimized local manga list, now unsorted and depends on the OS for the order of file

## [0.25.6]

### Fixed
- [tanoshi-web] fix web crash on single reader when using fit height
- 
## [0.25.5]

## Changed
- [tanoshi] use non random iv so url stay the same and browser can cache them
- [tanoshi-web] set max width to 768px on vertical mode

## [0.25.4]

## Changed
- [tanoshi] every image url now encrypted to verify they come from tanoshi
- [tanoshi] image proxy only receive encryped url
- [tanoshi] image proxy now use param instead of query
- [tanoshi] pages no longer a column in chapter, but its own table
- [tanoshi] local_url if page table for downloaded chapter later

## Fixed
- [tanoshi-web] page may show unordered and panic on reader
- [tanoshi-web] empty chapter no longer treated as error if both from db and sources is empty

## [0.25.3]

### Added
- [tanoshi] notification schema for testing
- [tanoshi-web] fetch telegram id on profile
- [tanoshi-web] add test telegram button on profile
- [tanoshi-web] retry button if image failed to load

## Fixed
- [tanoshi-web] page freeze when select fit option in paged mod
- 
## [0.25.2]

Nothing changes, this release to build for multiarch docker image

## [0.25.1]

### Added
- [tanoshi-util] add log utility for extensions
- [tanoshi] graceful shutdown, close database on server shutdown

### Changed
- [tanoshi] local sources manga list now sorted
- [tanoshi-web] frontend now force logout on unactivated server

### Fixed
- [tanoshi] fix local source duplicate list on `load more`
- [tanoshi] fix non folder or non cbz/cbr files not filtered

## [0.25.0]

### Added 

- [tanoshi] Periodic background updates
- [tanoshi] Telegram bot notification
- [tanoshi] Support local manga chapter as directory instead of archive files
- [tanoshi-web] Navigate to prev/next chapter at start/end of chapter
- [tanoshi-web] Loading spinner when change chapter and installing extenstion
- [tanoshi-util] implement `http_request` supporting http method other than  `GET`
- [tanoshi-lib] move `Request` and `Response` to `tanoshi-util`
- [tanoshi-vm] use `http_request` implementation from `tanoshi-util`

### Changed

- [tanoshi] Decouple db and graphql
- [tanoshi] Replace option with result
- [tanoshi] Check token expiration
- [tanoshi] `get_claims` return result
- [tanoshi] Establish database connection return result
- [tanoshi-web] Serialize and Deserialize from catalogue into local_storage instead of individual fields
- [tanoshi-web] Use `with_node` because `events:value` is deprecated
- [tanoshi] Implement clippy suggestion
- [tanoshi-web] Bigger fonts in input box
- [tanoshi-web] Use primary color for button inside topbar
- [tanoshi-vm] Reduce memory consumption by separating compile and runtime for extension
- [tanoshi-web] Reduce code duplication on `query.rs` 
- [tanoshi] library now default to sorted by title
- [tanoshi-vm] process will spawn task for concurrency

### Fixed

- [tanoshi-web] cover image brightness not lowered in catalogue if favorited
- [tanoshi] refresh all libraries
- [tanoshi-web] manga title wrap
- [tanoshi] failed to register first time because backend check non existent token

## [0.24.6] - 2021-08-03

### Fixed

- fix catalogue not fetch next page
- fix some stylings

## [0.24.5] - 2021-07-24

### Changed

- Switch from yarn to trunk
- Migrate from tailwind to sass

### Fixed

- Fix web won't load

## [0.24.4] - 2021-07-16

### Fixed

- fix bottombar showing in reader

## [0.24.3] - 2021-07-16

### Added

- show error as snackbar

### Changed

- reduce panic

## [0.24.2] - 2021-07-11

### Fixed

- fix panic when using local source

[Unreleased]: https://github.com/faldez/tanoshi/compare/v0.25.6...HEAD
[0.25.6]: https://github.com/faldez/tanoshi/compare/v0.25.5...v0.25.6
[0.25.5]: https://github.com/faldez/tanoshi/compare/v0.25.4...v0.25.5
[0.25.4]: https://github.com/faldez/tanoshi/compare/v0.25.3...v0.25.4
[0.25.3]: https://github.com/faldez/tanoshi/compare/v0.25.2...v0.25.3
[0.25.2]: https://github.com/faldez/tanoshi/compare/v0.25.1...v0.25.2
[0.25.1]: https://github.com/faldez/tanoshi/compare/v0.25.0...v0.25.1
[0.25.0]: https://github.com/faldez/tanoshi/compare/v0.24.6...v0.25.0
[0.24.6]: https://github.com/faldez/tanoshi/compare/v0.24.5...v0.24.6
[0.24.5]: https://github.com/faldez/tanoshi/compare/v0.24.4...v0.24.5
[0.24.4]: https://github.com/faldez/tanoshi/compare/v0.24.3...v0.24.4
[0.24.3]: https://github.com/faldez/tanoshi/compare/v0.24.2...v0.24.3
[0.24.2]: https://github.com/faldez/tanoshi/compare/v0.24.1...v0.24.2
