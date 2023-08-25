# `screencapturekit-rs` A Safe Rust Wrapper

## Introduction

ScreenCaptureKit is a high-performance screen capture framework for macOS applications. It provides fine-grained control to select and capture specific content on the screen, such as an app window, and is particularly useful in video conferencing apps where users can choose to share only part of their screen. This Rust wrapper aims to provide a safe and easy-to-use interface to the ScreenCaptureKit framework.
## Features

- **High Performance**: ScreenCaptureKit is performance-focused and leverages the power of Mac GPUs with a lower CPU overhead than existing capture methods [SDK Docs](https://developer.apple.com/videos/play/wwdc2022/10156/).
- **Fine-Grained Control**: With ScreenCaptureKit, you can specify the type of content you want to share or filter out. You can capture screen content from any combination of displays, applications, and windows as well as the audio that goes with it [SDK Docs](https://developer.apple.com/videos/play/wwdc2022/10156/).
- **Flexible Configuration**: ScreenCaptureKit supports a variety of developer controls, including pixel format, color space, frame rate, and resolution, and on the audio side, controls such as sample rate and channel count. All of these filters and configurations can be adjusted on the fly, allowing for more flexibility in application design [SDK Docs](https://developer.apple.com/videos/play/wwdc2022/10156/).
- **Privacy**: ScreenCaptureKit is built with privacy in mind, providing global privacy safeguards for all applications using the framework. The framework will require consent before capturing video and audio content, and the choice will be stored in the Screen Recording privacy setting in system preferences [SDK Docs](https://developer.apple.com/videos/play/wwdc2022/10156/).

## Usage

To use this wrapper, you need to follow a few steps:
TBD
## Dependencies

The ScreenCaptureKit Safe Rust Wrapper depends on the following:
- **macOS 12.3 or later**: ScreenCaptureKit is available in macOS 12.3 and later versions [SDK Docs](https://developer.apple.com/documentation/screencapturekit?language=objc)
- The objc family of crates (Authored by: @SSheldon): 
	- [objc](https://docs.rs/objc/)
	- [objc_id](https://docs.rs/objc_id)
	- [objc-foundation](https://docs.rs/objc_foundation)
	- [objc-encode](https://docs.rs/objc_encode)
	- [block](https://docs.rs/block)
	- [dispatch](https://docs.rs/block)

# License(s)

This project is licensed under the [GNU Lesser General Public License v3.0 or later](LICENSE). 

# Contributing <!-- Include this section if you want to be open to receive contributions -->

TBD

We appreciate contributions in the form of bug reports, fixes, feature requests etc, and will review them as time permits. Please keep in mind that this project is experimental and the maintainers' time is limited.

# Creators & Maintainers

TBD
