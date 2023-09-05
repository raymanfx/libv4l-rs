# Changelog

The project adheres to semantic versioning, as do most Rust projects.
Changelog items are created for major and minor releases, but not bugfix ones for now.

Currently, the focus is on implementing the high-level V4L2 single-planar API.
Multi-planar capture will not be targeted in the near future unless someone else starts working on it.


## [0.13.1] - 2023-05-26
### Added
- Basic multi-planar streaming support

## [0.14.0] - 2023-05-13
### Added
- Expose raw file descriptor of streams through `Stream::handle()`
### Changed
- Updated `bindgen` dependency to 0.65.1
### Fixed
- Use proper C FFI struct field for `Integer64` controls
- Fix example in README.md to account for the negotiated pixelformat

## [0.13.1] - 2022-12-08
### Fixed
- Do not block when the device is disconnected
  - This is achieved by using the non-blocking file descriptor API internally
  - The outside-facing API is the same for now
  - Can be used as a foundation for language-level async support in the future

## [0.13.0] - 2022-05-19
### Added
- Handling of boolean and button controls
- MJPG (consumer class hardware) support in `glium` example
### Changed
- Simplified examples, removing clap argument parsing
- Unified Value/Value64 control types into a single Integer enum variant
### Fixed
- MUSL libc compatibility
- Android cross compilation

## [0.12.2] - 2021-19-01
### Fixed
- Avoid dropping frames by queuing all buffers on stream start

## [0.12.1] - 2021-05-01
### Fixed
- Update the buffer index for output streams
- Honor the bytesused field for compressed frames in output streams

## [0.12.0] - 2021-01-07
### Changed
- Depend on `0.2.0` sys packages to ship bindgen 0.56.0

## [0.11] - 2020-12-31
### Added
- Global context struct
  - Used to enumerate devices
- Single, unified device struct
  - Implement Capture / Output capabilities as traits
- MMAP support for output streams (see `stream_forward_mmap` example)
### Changed
- Fine grained buffer access and handling for streams

## [0.10] - 2020-08-26
### Added
- Output device support!
  - Just the single-planar API for now.
  - Only write() I/O, no mmap or other buffer types.

## [0.9] - 2020-08-05
### Added
- New Handle type for passing around device handles
  - You can now stream buffers while changing device controls at the same time! Handles are
    thread safe (Arc) by default.
- New StreamItem type introduced to better model stream semantics
  - An item only lives up to the point in time where you query the next item from the stream.
### Removed
- Removed buffer arenas from public API

## [0.8] - 2020-08-01
### Added
- New prelude module
### Changed
- I/O module reorganization
- Renamed BufferManagers to Arenas
- Use a single Buffer struct for all I/O streams
### Removed
- Removed the 'get_' prefix on getters

## [0.7] - 2020-07-06
### Added
- Device control get/set support
- New QueryDevice trait
  - Implemented for all types which implement Device
  - Allows for querying properties such as supported frame times and controls

## [0.6] - 2020-07-04
### Added
- Device control query support
### Changed
- Use v4l2 bindings by default

## [0.5] - 2020-05-14
### Changed
- Device API refactoring
  - We only support the V4L2 capture API for now
  - Overlay, Output and friends will be added in future releases

## [0.4] - 2020-05-03
### Added
- Streaming I/O (userptr)
### Changed
- Optional libv4l vs v4l2 FFI dependencies

## [0.3] - 2020-05-02
### Added
- Device buffer abstraction
  - Streaming I/O (mmap)

## [0.2] - 2020-05-01
### Added
- Device list with capability querying
- Device abstraction
  - Format enumeration
  - Format getter/setter
  - Parameter getter/setter

## [0.1] - 2020-04-30
### Added
- v4l-sys bindings
- I/O codes (VIDIOC_*)
