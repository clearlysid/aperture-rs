use lazy_static::lazy_static;
use rand::Rng;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io;
use std::process::Command;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::process::Command as TokioCommand;
use tokio::time::sleep;

// Path to Aperture Binary
const BIN: &str = "/Users/siddharth/code/aperture/src/bin/aperture";

#[derive(Serialize)]
struct CropArea {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn get_random_id() -> String {
    let random_number: u64 = rand::thread_rng().gen();
    let id = format!("{:x}", random_number);
    id.chars().take(13).collect()
}

fn supports_hevc_hardware_encoding() -> bool {
    let output = Command::new("sysctl")
        .args(&["-n", "machdep.cpu.brand_string"])
        .output()
        .expect("Failed to get CPU info");

    let cpu_model = String::from_utf8_lossy(&output.stdout);

    // All Apple silicon Macs support HEVC hardware encoding.
    if cpu_model.starts_with("Apple ") {
        return true;
    }

    let re = regex::Regex::new(r#"Intel.*Core.*i\d+-(\d)"#).unwrap();
    if let Some(captures) = re.captures(&cpu_model) {
        if let Ok(generation) = captures[1].parse::<u32>() {
            // Intel Core generation 6 or higher supports HEVC hardware encoding
            return generation >= 6;
        }
    }

    false
}

pub async fn screens() -> Result<Value, Box<dyn std::error::Error>> {
    let output = TokioCommand::new(BIN)
        .args(&["list", "screens"])
        .output()
        .await?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let result = serde_json::from_str(&stderr)?;
    Ok(result)
}

pub async fn audio_devices() -> Result<Value, Box<dyn std::error::Error>> {
    let output = TokioCommand::new(BIN)
        .args(&["list", "audio-devices"])
        .output()
        .await?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let result = serde_json::from_str(&stderr)?;
    Ok(result)
}

lazy_static! {
    static ref VIDEO_CODECS: Vec<[&'static str; 2]> = {
        let mut codecs = vec![
            ["h264", "H264"],
            ["proRes422", "Apple ProRes 422"],
            ["proRes4444", "Apple ProRes 4444"],
        ];

        if supports_hevc_hardware_encoding() {
            codecs.push(["hevc", "HEVC"]);
        }

        codecs
    };
}

pub fn video_codecs() -> &'static Vec<[&'static str; 2]> {
    &VIDEO_CODECS
}

struct Aperture {
    process_id: String,
    recorder: Option<std::process::Child>,
    tmp_path: Option<NamedTempFile>,
}

impl Aperture {
    fn new() -> Self {
        Aperture {
            process_id: "".into(),
            recorder: None,
            tmp_path: None,
        }
    }

    async fn start_recording(
        &mut self,
        screen_id: u32,
        fps: u32,
        crop_area: CropArea,
        show_cursor: bool,
        highlight_clicks: bool,
        audio_device_id: String,
        video_codec: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.process_id = get_random_id();

        if self.recorder.is_some() {
            return Err("Call `stop_recording()` first".into());
        }

        self.tmp_path = Some(NamedTempFile::new()?);
        let tmp_path = self
            .tmp_path
            .as_ref()
            .unwrap()
            .path()
            .to_string_lossy()
            .to_string();

        let recorder_options = json!({
            "destination": &tmp_path,
            "framesPerSecond": fps.to_string(),
            "showCursor": show_cursor,
            "highlightClicks": highlight_clicks,
            "screenId": screen_id,
            "audioDeviceId": audio_device_id,
            "cropRect": crop_area,
            "videoCodec": video_codec.unwrap_or("h264".into())
        });

        // print recordor options
        println!("recorder_options: {}", recorder_options);

        return Ok(());
    }

    pub async fn wait_for_event(
        &self,
        name: &str,
        parse: Option<fn(&str) -> Option<String>>,
    ) -> Option<String> {
        let output = TokioCommand::new(BIN)
            .args(&[
                "events",
                "listen",
                "--process-id",
                &self.process_id,
                "--exit",
                name,
            ])
            .output()
            .await
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if let Some(parse_fn) = parse {
            parse_fn(&stdout)
        } else {
            None
        }
    }

    pub async fn send_event(
        &self,
        name: &str,
        parse: Option<fn(&str) -> Option<String>>,
    ) -> Option<String> {
        let output = TokioCommand::new(BIN)
            .args(&["events", "send", "--process-id", &self.process_id, name])
            .output()
            .await
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if let Some(parse_fn) = parse {
            parse_fn(&stdout)
        } else {
            None
        }
    }

    fn throw_if_not_started(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.recorder.is_none() {
            Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                "Call `.start_recording()` first",
            )))
        } else {
            Ok(())
        }
    }

    async fn pause(self) {
        self.throw_if_not_started();
        self.send_event("pause", None).await;
    }

    async fn resume(self) {
        self.throw_if_not_started();
        self.send_event("resume", None).await;
    }

    // async fn isPaused(self) -> Result<bool, Box<dyn std::error::Error>> {
    //     self.throw_if_not_started();
    //     let value = self
    //         .send_event("isPaused", Some(|value| value == "true"))
    //         .await
    //         .unwrap_or(false); // Default to false if the event value is not available

    //     Ok(value)
    // }
}

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

    println!("File is ready");
    sleep(Duration::from_secs(5)).await;

    // let fp = recorder.stop_recording().await.unwrap();
    // fs::rename(fp, "recording.mp4").unwrap();
    println!("Video saved in the current directory");
}
