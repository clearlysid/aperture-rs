use aperture::{audio_devices, screens, video_codecs, Aperture, CropArea};
use std::fs;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let screens = screens().await.unwrap();
    println!("Screens: {:?}", screens);

    let audio_devices = audio_devices().await.unwrap();
    println!("Audio_devices: {:?}", audio_devices);

    let video_codecs = video_codecs();
    println!("Video_codecs: {:?}", video_codecs);

    println!("Preparing to record for 5 seconds");
    let mut recorder = Aperture::new();

    recorder
        .start_recording(
            1,  // screen_id ("BuiltInRetinaDisplay")
            30, // fps
            CropArea {
                // crop_area
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            true,                                  // show_cursor
            false,                                 // highlight_clicks
            "BuiltInMicrophoneDevice".to_string(), // audio_device_id
            Some(("hevc").to_string()),            // video_codec
        )
        .await
        .unwrap(); // Handle the Result accordingly
    println!("Recording started");

    sleep(Duration::from_secs(5)).await;

    let fp = recorder.stop_recording().await.unwrap();
    fs::rename(fp, "recording.mp4").unwrap();
    println!("Video saved in the current directory");
}
