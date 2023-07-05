# aperture-rs

> Record the screen on macOS from Rust

## TODOs

-   [ ] Make binary path relative
-   [ ] Ensure optional props are passed into recording options correctly
-   [ ] Bundle binary into crate
-   [ ] Improve Readme and documentation
-   [ ] Publish to Cargo

## Install (Coming Soon)

Add to your `Cargo.toml`

```toml
aperture = "0.1"
```

_Requires macOS 10.13 or later._

## Usage

```rust
use aperture::Aperture;

fn main() {
	// Initialize recorder
	let mut recorder = Aperture::new();

	// Start recording
	recorder
		.start_recording(
			0, // screen_id
			60, // fps
			true, // show_cursor
			false, // highlight_clicks
			Some(("hvc1").to_string()), // video_codec
			Some("BuiltInMicrophoneDevice".to_string()), // audio_device_id
		)
		.await
		.unwrap();

	// Wait 10 seconds
	sleep(Duration::from_secs(10)).await;

	// Stop recording
	let final_video = recorder.stop_recording().await.unwrap();

	// Print path to final video.
	println!("Video saved to: {}", final_video);
}
```

See [`src/example.rs`](src/example.rs) for a working demo.

## API

1. List Screens
2. List Audio Devices
3. List Supported Codecs

## Arguments

1. fps
2. CropArea
3. show_cursor
4. highlight_clicks
5. screenId
6. audioDeviceId
7. videoCodec

## Why

Aperture was originally made for [Kap](https://github.com/wulkano/kap). Under the hood it uses a Swift Script that records the screen using the pr[AVFoundation framework](https://developer.apple.com/av-foundation/) by Apple.

#### But you can use `ffmpeg -f avfoundation...`

You can, but the performance is terrible:

##### Recording the entire screen with `ffmpeg -f avfoundation -i 1 -y test.mp4`:

![ffmpeg](https://cloud.githubusercontent.com/assets/4721750/19214740/f823d4b6-8d60-11e6-8af3-4726146ef29a.jpg)

##### Recording the entire screen with Aperture:

![aperture](https://cloud.githubusercontent.com/assets/4721750/19214743/11f4aaaa-8d61-11e6-9822-4e83bcdfab24.jpg)

## Related

-   [Aperture](https://github.com/wulkano/Aperture): Swift framework used in this package.
-   [aperture-node](https://github.com/wulkano/aperture-node/tree/main): Node bindings for use in JS/TS projects.
-   [Kap](https://github.com/wulkano/Kap): The screen-recording app that started it all.
