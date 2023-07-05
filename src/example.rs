use aperture::{audio_devices, screens, video_codecs, Aperture, Options};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let screens = screens().await.unwrap();
    println!("Screens: {:?}", screens);

    let audio_devices = audio_devices().await.unwrap();
    println!("Audio Devices: {:?}", audio_devices);

    let video_codecs = video_codecs();
    println!("Video Codecs: {:?}", video_codecs);

    println!("------------------------------------");

    println!("Preparing to record for ~10 seconds");
    let mut recorder = Aperture::new();

    let recorder_options = Options {
        screen_id: 1,
        fps: 30,
        show_cursor: true,
        highlight_clicks: true,
        video_codec: None,
        audio_device_id: None,
        crop_area: None,
    };

    recorder.start_recording(recorder_options).await.unwrap();
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
