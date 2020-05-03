# Roadmap

Currently, the focus is on implementing the high-level V4L2 single-planar API.
Multi-planar capture will not be targeted in the near future unless someone else starts working on it.



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

