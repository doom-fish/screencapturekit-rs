> #### API for desktop recording
> If you’re looking for a hosted desktop recording API, consider checking out [Recall.ai](https://www.recall.ai/product/desktop-recording-sdk), an API that lets you record Zoom, Google Meet, Microsoft Teams, In-person meetings, and more.


# screencapturekit-rs

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


## Contributors

<!-- readme: collaborators,contributors -start -->
<table>
	<tbody>
		<tr>
            <td align="center">
                <a href="https://github.com/1313">
                    <img src="https://avatars.githubusercontent.com/u/1427038?v=4" width="100;" alt="1313"/>
                    <br />
                    <sub><b>Per Johansson</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/krzykro2">
                    <img src="https://avatars.githubusercontent.com/u/6817875?v=4" width="100;" alt="krzykro2"/>
                    <br />
                    <sub><b>Kris Krolak</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/iparaskev">
                    <img src="https://avatars.githubusercontent.com/u/23037402?v=4" width="100;" alt="iparaskev"/>
                    <br />
                    <sub><b>Iason Paraskevopoulos</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/Pranav2612000">
                    <img src="https://avatars.githubusercontent.com/u/20909078?v=4" width="100;" alt="Pranav2612000"/>
                    <br />
                    <sub><b>Pranav Joglekar</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/uohzxela">
                    <img src="https://avatars.githubusercontent.com/u/4747352?v=4" width="100;" alt="uohzxela"/>
                    <br />
                    <sub><b>Alex Jiao</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/aizcutei">
                    <img src="https://avatars.githubusercontent.com/u/20311560?v=4" width="100;" alt="aizcutei"/>
                    <br />
                    <sub><b>Charles</b></sub>
                </a>
            </td>
		</tr>
		<tr>
            <td align="center">
                <a href="https://github.com/tokuhirom">
                    <img src="https://avatars.githubusercontent.com/u/21084?v=4" width="100;" alt="tokuhirom"/>
                    <br />
                    <sub><b>Tokuhiro Matsuno</b></sub>
                </a>
            </td>
            <td align="center">
                <a href="https://github.com/bigduu">
                    <img src="https://avatars.githubusercontent.com/u/18681616?v=4" width="100;" alt="bigduu"/>
                    <br />
                    <sub><b>bigduu</b></sub>
                </a>
            </td>
		</tr>
	<tbody>
</table>
<!-- readme: collaborators,contributors -end -->
