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
See [ROADMAP.md](https://github.com/raymanfx/libv4l-rs/blob/master/ROADMAP.md)
