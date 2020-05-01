# Safe video4linux (v4l) bindings

[![crates.io](https://img.shields.io/crates/v/v4l.svg)](https://crates.io/crates/v4l)
[![license](https://img.shields.io/github/license/raymanfx/libv4l-rs)](https://github.com/raymanfx/libv4l-rs/blob/master/LICENSE.txt)
[![Build Status](https://travis-ci.org/raymanfx/libv4l2-rs.svg?branch=master)](https://travis-ci.org/raymanfx/libv4l2-rs)

This crate provides safe bindings to the libv4l* stack consisting of:
 * libv4l1
 * libv4l2
 * libv4lconvert

Thus, it enables you to capture frames from camera devices on Linux using common formats such as RGB3, even if the camera does not support it natively.

## Goals
This crate shall provide the v4l-sys package to enable full (but unsafe) access to libv4l\*.
On top of that, there will be a high level, more idiomatic API to use video capture devices in Linux.

There will be simple utility applications to list devices and capture frames.
A minimalistic OpenGL/Vulkan viewer to display frames is planned for the future.

## Roadmap
Currently, the focus is on implementing the high-level V4L2 single-planar API.
Multi-planar capture will not be targeted in the near future unless someone else starts working on it.

#### 0.1 (released)
 * v4l-sys bindings
 * I/O codes (VIDIOC_*)

#### 0.2 (released)
 * Device list with capability querying
 * Device abstraction
     * Format enumeration
     * Format getter/setter
     * Parameter getter/setter

#### 0.3
 * Device buffer abstraction
     * Streaming I/O (mmap, userptr, dmabuf)
     * Capture method for Device
