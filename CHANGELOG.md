# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- [tanoshi] major refactor to clean arch
- [tanoshi-web] change edit chapter action icon

## Added
- [tanoshi] support notification with Gotify
- [tanoshi] support cb7 format
- [tanoshi] config to disable create database if missing
- [tanoshi] server-side image cache
- [tanoshi] delete chapter api
- [tanoshi] full chapters' source sync, remove chapters no longer in source

## [0.28.1]
### Fixed
- [tanoshi-desktop] page not loaded

## [0.28.0]
### Added
- [tanoshi] MyAnimeList tracking
- [tanoshi] AniList tracking

### Changed
- [tanoshi] tracker and notifier move to their own crates
- [tanoshi] chapter update worker will revert to insert all chapter and replace on conflict, but still only notify new chapter
- [tanoshi] docker image use bookworm-slim
- [tanoshi] use rayon `.par_iter` when possible
  
### Fixed
- [tanoshi] archive with folder cannot be read
- [tanoshi] special character in filename return error

## [0.27.1]
### Added
- [tanoshi] add link to chapter on chapter update notification if `BASE_URL` is set
  
### Changed
- [tanoshi] clean download file name is now done regardless of OS
- [tanoshi-web] increas preload by 1 on continous reader
  
### Fixed
- [tanoshi-web] filter input checkbox state not changed

## [0.27.0]
### Changed
- [tanoshi] extension is back using dynamic library instead of webassembly or javascript

## [0.26.1]
### Changed
- [tanoshi-vm] add timeout to async operations

## [0.26.0]
### Added
- [tanoshi] source filter and settings
- [tanoshi] multiple folder for local sources
  ```yaml
  # single local source
  local_path: .\manga
  # multiple local sources
  local_path:
    - name: Local
      path: .\manga
    - name: Public
      path: .\test\data\manga
  ```

### Changed
- [tanoshi] extension now using and ported to javascript
- [tanoshi] pages no longer cached to database
- [tanoshi] downloaded manga path moved to table chapter
- [tanoshi-web] on continous reader, scrolled to bottom automatically marked as last page

## [0.25.22]

### Added
- [tanoshi] library categories
- [tanoshi] link field in manga
- [tanoshi-web] external link button in manga detail
- [tanoshi-web] add logout button
- [tanoshi-web] page slider
- [tanoshi-web] changelog, github, website and discord link on settings page
- [tanoshi] image proxy has referer query
- [tanoshi] image proxy now forward every header to upstream

### Changed
- [tanoshi-desktop] topbar is now white
- [tanoshi-desktop] slight layout changes
- [tanoshi-web] updates chapter number now sorted descending
- [tanoshi-web] manga detail on dual column layout now scrollable
- [tanoshi-web] redesign action button in manga detail page
- [tanoshi-web] zoom button moved to bottom bar
- [tanoshi-web] page slider direction follow reader direction
- [tanoshi-web] reorganize `more` page
- [tanoshi-vm] remove async from extension thread
- [tanoshi-vm] add non async function to extension bus

### Removed
- [tanoshi-web] manage downloads page

## [0.25.21]

### Added
- Download chapters from external sources
- Add details to local manga with `details.json` inside series folder. All values are optional
  ```
  {
    "title": "An Interesting Manga",
    "author: ["Author 1", "Author 2"],
    "genre: ": ["Romance", "Action"],
    "status": "Ongoing",
    "description": "This manga is so interesting",
    "cover_path": "relative/path/from/root/series/folder/to/thumbail.jpg",
  }
  ```
- Automatically download new chapters on update. Enable with set `auto_download_chapters: true` on `config.yml`
- Desktop version built with tauri if you don't plan to host it

### Changes
- Few icon changes
- Desktop layout
- Performance improvement for library and manga details page
- Opening first page will update history
- Zoom button move to bottom right in vertical
- Hide bottombar inside settings page
- Topbar and bottombar autohide on reader, tap image or middle screen to bring back

## [0.25.20]
### Added
- Zoom in and zoom out button in reader

### Changed
- Replace some text button with icon button
- Auto close snackbar 
- animate.css now bundled
- Reader background color set for body instead of reader element

## [0.25.19]

### Added
- Fade in animation for manga cover in library and catalogue
- More breakpoint for manga grid in library and catalogue
- Keyboard navigation

### Changed
- Reader now only preload few pages ahead and back
- Load image from disk now async 
- Modal width max at 768px
- Animation is now faster
- Appearance setting save on change
- Title in manga page now bold

### Fixed

## [0.25.18]

### Added
- [tanoshi-web] restore last page read on continuous mode
- [tanoshi] installedSource query has check update param

### Fixed
- [tanoshi-web] large header on firefox
- [tanoshi-web] next chapter doesn't scroll to top on continous mode

### Changed
- [tanoshi-web] page indicator style changes
- [tanoshi] move from warp to axum
- [tanoshi-vm] extension now loaded then dropped every call
- [tanoshi] limit sqlite connection to 5 with 1 minute idle timeout and 3 minute max lifetime
- [tanoshi] image proxy now serve stream data


## [0.25.17]

### Added
- [tanoshi] pushover notification
- [tanoshi-web] continuous reader pages have default height when loading
- [tanoshi-web] global search
- [tanoshi-web] filter and sort manga in library

### Changed
- [tanoshi] tanoshi will no longer compile wasm from extension repo, instead download precompiled extension
- [tanoshi] use dylib engine instead of univerval engine reduce memory usage
- [tanoshi] wasm extension now compiled using llvm instead of cranelift
- [tanoshi] extension process no longer spawn task, reduce chances unreachable error

## [0.25.16]

### Fixed
- [tanoshi-web] global sort settings is ignored

## [0.25.15]

### Added
- [tanoshi-web] add global and per manga chapter sort and filter settings

### Changed
- [tanoshi-web] reader settings in settings page automatically save without click apply

## [0.25.14]

### Added
- [tanoshi-web] select all and deselect all in chapter selection

### Changed
- [tanoshi] mark chapter as read always update is_complete as true

## [0.25.13]

### Fixed
- [tanoshi] fix user_history migration script

## [0.25.12]

### Added
- [tanoshi-web] unread chapters badges for manga in library
- [tanoshi] unread chapter count for manga
- [tanoshi] `is_complete` field to set a chapter is completely read
- [tanoshi] extension update notification
- [tanoshi-web] reader open from last page read in paged mode
- [tanoshi-web] source is shown in manga detail page
- [tanoshi-web] filter chapter by read or unread
- [tanoshi-web] sort chapter by read at or number
- [tanoshi-web] resume button

### Changed
- [tanoshi] mark chapter as read set last_page to last page of a chapter if availavle and mark `is_complete` as true
- [tanoshi-web] opening first page won't update history
- [tanoshi] interval between chapter refresh in periodic update is now 500ms

### Fixed
- [tanoshi-web] current page reset to zero after last page
- [tanoshi-web] double spread image on double paged reader not on center

## [0.25.11]

### Added
- [tanoshi-web] go to manga detail from history and update page
- [tanoshi-web] search in library
- [tanoshi-vm] log extension load time
- [tanoshi-vm] show which command has receiver dropped error

### Changed
- [tanoshi] `sourceId` in manga is replaced with `source`
- [tanoshi-vm] source detail cache in memory, no need to call webassemby function for detail

### Fixed
- [tanoshi-web] text input have full border radius
- [tanoshi-web] theme not changing when prefres-color-scheme change
- [tanoshi-web] fit setting not set on certain manga reader settings
- [tanoshi] extension file removed after uninstall
- [tanoshi] refresh manga from browsing catalogue results in wrong chapters for manga

## [0.25.10]

### Added
- [tanoshi-web] appearance settings, manually change theme
- [tanoshi-web] prefer color scheme event listener
- [tanoshi] add health check endpoint

### Changed
- [tanoshi-web] checkbox color now more gray and have primary color when checked
- [tanoshi-web] reader setting don't use separate struct
- [tanoshi] periodic updates now have 100ms delay

## [0.25.9]

### Changed
- [tanoshi] revert libarchive-rs revision before custom read callback
 
## [0.25.8]

### Added
- [tanoshi-web] scanlator now shown in chapter list
- [tanoshi] mark chapter as read and mark chapter as unread
- [tanoshi-web] show version in settings

### Changed
- [tanoshi] local get_chapter now sorted
- [tanoshi-web] reduce bottom paddding on reader settings modal

### Fixed
- [tanoshi-web] fix typo intial -> initial
  
## [0.25.7]

### Added
- [tanoshi] check for update every 24 hours and send notification to admins if found
- [tanoshi-lib] Version struct now on tanoshi-lib, `verion` field in `Source` now is Version struct
- [tanoshi-lib] add lib_version to `Source` to identify `tanoshi-lib` version is used

### Changed
- [tanoshi] optimized local manga list, now unsorted and depends on the OS for the order of file
- [tanoshi-web] use wasm-opt=4 and build with `--release` for release

## [0.25.6]

### Fixed
- [tanoshi-web] fix web crash on single reader when using fit height
  
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

[Unreleased]: https://github.com/faldez/tanoshi/compare/v0.28.1...HEAD
[0.28.1]: https://github.com/faldez/tanoshi/compare/v0.28.0...v0.28.1
[0.28.0]: https://github.com/faldez/tanoshi/compare/v0.27.1...v0.28.0
[0.27.1]: https://github.com/faldez/tanoshi/compare/v0.27.0...v0.27.1
[0.27.0]: https://github.com/faldez/tanoshi/compare/v0.26.1...v0.27.0
[0.26.1]: https://github.com/faldez/tanoshi/compare/v0.26.0...v0.26.1
[0.26.0]: https://github.com/faldez/tanoshi/compare/v0.25.22...v0.26.0
[0.25.22]: https://github.com/faldez/tanoshi/compare/v0.25.21...v0.25.22
[0.25.21]: https://github.com/faldez/tanoshi/compare/v0.25.20...v0.25.21
[0.25.20]: https://github.com/faldez/tanoshi/compare/v0.25.19...v0.25.20
[0.25.19]: https://github.com/faldez/tanoshi/compare/v0.25.18...v0.25.19
[0.25.18]: https://github.com/faldez/tanoshi/compare/v0.25.17...v0.25.18
[0.25.17]: https://github.com/faldez/tanoshi/compare/v0.25.16...v0.25.17
[0.25.16]: https://github.com/faldez/tanoshi/compare/v0.25.15...v0.25.16
[0.25.15]: https://github.com/faldez/tanoshi/compare/v0.25.14...v0.25.15
[0.25.14]: https://github.com/faldez/tanoshi/compare/v0.25.13...v0.25.14
[0.25.13]: https://github.com/faldez/tanoshi/compare/v0.25.12...v0.25.13
[0.25.12]: https://github.com/faldez/tanoshi/compare/v0.25.11...v0.25.12
[0.25.11]: https://github.com/faldez/tanoshi/compare/v0.25.10...v0.25.11
[0.25.10]: https://github.com/faldez/tanoshi/compare/v0.25.9...v0.25.10
[0.25.9]: https://github.com/faldez/tanoshi/compare/v0.25.8...v0.25.9
[0.25.8]: https://github.com/faldez/tanoshi/compare/v0.25.7...v0.25.8
[0.25.7]: https://github.com/faldez/tanoshi/compare/v0.25.6...v0.25.7
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
