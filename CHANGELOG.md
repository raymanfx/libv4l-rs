# Changelog

Currently, the focus is on implementing the high-level V4L2 single-planar API.
Multi-planar capture will not be targeted in the near future unless someone else starts working on it.



#### 0.9 (released)

> * New Handle type for passing around device handles
>   * You can now stream buffers while changing device controls at the same time! Handles are
>     thread safe (Arc) by default.
> * New StreamItem type introduced to better model stream semantics
>   * An item only lives up to the point in time where you query the next item from the stream.
> * Removed buffer arenas from public API

#### 0.8 (released)

> * New prelude module
> * Removed the 'get_' prefix on getters
> * I/O module reorganization
> * Renamed BufferManagers to Arenas
> * Use a single Buffer struct for all I/O streams

#### 0.7 (released)

> * Device control get/set support
> * New QueryDevice trait
>   * Implemented for all types which implement Device
>   * Allows for querying properties such as supported frame times and controls

#### 0.6 (released)

> * Use v4l2 bindings by default
> * Device control query support

#### 0.5 (released)

> * Device API refactoring
>   * We only support the V4L2 capture API for now
>   * Overlay, Output and friends will be added in future releases

#### 0.4 (released)

> * Streaming I/O (userptr)
> * Optional libv4l vs v4l2 FFI dependencies

#### 0.3 (released)
> * Device buffer abstraction
>   * Streaming I/O (mmap)

#### 0.2 (released)
> * Device list with capability querying
> * Device abstraction
>   * Format enumeration
>   * Format getter/setter
>   * Parameter getter/setter

#### 0.1 (released)
> * v4l-sys bindings
> * I/O codes (VIDIOC_*)

