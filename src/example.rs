use aperture::{audio_devices, screens, video_codecs, Aperture, CropArea};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let screens = screens().await.unwrap();
    println!("Screens: {:?}", screens);

    let audio_devices = audio_devices().await.unwrap();
    println!("Audio_devices: {:?}", audio_devices);

    let video_codecs = video_codecs();
    println!("Video_codecs: {:?}", video_codecs);

    println!("Preparing to record for ~10 seconds");
    let mut recorder = Aperture::new();

    recorder
        .start_recording(
            0,                          // screen_id ("BuiltInRetinaDisplay")
            30,                         // fps
            true,                       // show_cursor
            false,                      // highlight_clicks
            Some(("hvc1").to_string()), // video_codec
            Some("BuiltInMicrophoneDevice".to_string()), // audio_device_id
                                        // None,                                  // crop_area
        )
        .await
        .unwrap();
    println!("Recording started");

    sleep(Duration::from_secs(10)).await;

    let final_video = recorder.stop_recording().await.unwrap();
    println!("Video saved to: {}", final_video);

    //  open the video in the default video player
    std::process::Command::new("open")
        .arg(final_video)
        .spawn()
        .expect("Failed to open video");
}
