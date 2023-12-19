# screencapturekit-rs
<!-- ALL-CONTRIBUTORS-BADGE:START - Do not remove or modify this section -->
[![All Contributors](https://img.shields.io/badge/all_contributors-4-orange.svg?style=flat-square)](#contributors-)
<!-- ALL-CONTRIBUTORS-BADGE:END -->

## Introduction

ScreenCaptureKit is a high-performance screen capture framework for macOS applications.
It provides fine-grained control to select and capture specific content on the screen,
such as an app window, and is particularly useful in video conferencing apps where
users can choose to share only part of their screen. This Rust wrapper aims to
provide a safe and easy-to-use interface to the ScreenCaptureKit framework.

## Features

- **High Performance**: ScreenCaptureKit is performance-focused and leverages
  the power of Mac GPUs with a lower CPU overhead than existing capture methods.
- **Fine-Grained Control**: With ScreenCaptureKit, you can specify the
  type of content you want to share or filter out. You can capture screen content
  from any combination of displays, applications, and windows
  as well as the audio that goes with it.
- **Flexible Configuration**: ScreenCaptureKit supports a variety of developer controls,
  including pixel format, color space, frame rate, and resolution,
  and on the audio side, controls such as sample rate and channel count.
  All of these filters and configurations can be adjusted on the fly,
  allowing for more flexibility in application design.

- **Privacy**: ScreenCaptureKit is built with privacy in mind,
  providing global privacy safeguards for all applications using the framework.
  The framework will require consent before capturing video and audio content,
  and the choice will be stored in the Screen Recording privacy setting in
  system preferences.

[More information](https://developer.apple.com/videos/play/wwdc2022/10156/).

## Usage

To use this wrapper, you need to follow a few steps:

```rust
// TBD
```

## Dependencies

The ScreenCaptureKit Safe Rust Wrapper depends on the following:

- **macOS 12.3 or later**: ScreenCaptureKit is available in macOS 12.3 and later
  versions [SDK Docs](https://developer.apple.com/documentation/screencapturekit?language=objc).
- **`objc` family of crates**:
  [objc](https://docs.rs/objc/),
  [objc_id](https://docs.rs/objc_id),
  [objc-foundation](https://docs.rs/objc_foundation),
  [block](https://docs.rs/block),
  [dispatch](https://docs.rs/block).

## License

Licensed under either of

- Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE)
- MIT license [LICENSE-MIT](LICENSE-MIT)

at your option.

## Contributing

We appreciate contributions in the form of bug reports,
fixes, feature requests etc, and will review them as time permits.
Please keep in mind that this project is experimental and the
maintainers' time is limited.




## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):


<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="http://64p.org/"><img src="https://avatars.githubusercontent.com/u/21084?v=4?s=100" width="100px;" alt="Tokuhiro Matsuno"/><br /><sub><b>Tokuhiro Matsuno</b></sub></a><br /><a href="https://github.com/svtlabs/screencapturekit-rs/commits?author=tokuhirom" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/bigduu"><img src="https://avatars.githubusercontent.com/u/18681616?v=4?s=100" width="100px;" alt="bigduu"/><br /><sub><b>bigduu</b></sub></a><br /><a href="https://github.com/svtlabs/screencapturekit-rs/commits?author=bigduu" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://doom.fish"><img src="https://avatars.githubusercontent.com/u/1427038?v=4?s=100" width="100px;" alt="Per Johansson"/><br /><sub><b>Per Johansson</b></sub></a><br /><a href="https://github.com/svtlabs/screencapturekit-rs/commits?author=1313" title="Code">💻</a> <a href="#ideas-1313" title="Ideas, Planning, & Feedback">🤔</a> <a href="#maintenance-1313" title="Maintenance">🚧</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.aizcutei.com"><img src="https://avatars.githubusercontent.com/u/20311560?v=4?s=100" width="100px;" alt="Charles"/><br /><sub><b>Charles</b></sub></a><br /><a href="https://github.com/svtlabs/screencapturekit-rs/commits?author=aizcutei" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
