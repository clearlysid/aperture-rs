use lazy_static::lazy_static;
use rand::Rng;
use serde::Serialize;
use serde_json::{json, Value};
use std::io;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, Duration};
use url::Url;

const APERTURE_BINARY: &str = "/Users/siddharth/code/aperture/src/bin/aperture";

#[derive(Serialize)]
pub struct CropArea {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
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
    let output = TokioCommand::new(APERTURE_BINARY)
        .args(&["list", "screens"])
        .output()
        .await?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let result = serde_json::from_str(&stderr)?;
    Ok(result)
}

pub async fn audio_devices() -> Result<Value, Box<dyn std::error::Error>> {
    let output = TokioCommand::new(APERTURE_BINARY)
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

pub struct Aperture {
    process_id: String,
    recorder: Option<std::process::Child>,
    tmp_path: Option<PathBuf>,
}

impl Aperture {
    pub fn new() -> Self {
        Aperture {
            process_id: "".into(),
            recorder: None,
            tmp_path: None,
        }
    }

    pub async fn start_recording(
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

        self.tmp_path = Some(NamedTempFile::new()?.into_temp_path().with_extension("mp4"));

        let tmp_path = self.tmp_path.as_ref().unwrap();

        let recorder_options = json!({
            "destination": Url::from_file_path(&tmp_path).unwrap().to_string(),
            "framesPerSecond": fps,
            "showCursor": show_cursor,
            "highlightClicks": highlight_clicks,
            "screenId": screen_id,
            "audioDeviceId": audio_device_id,
            "cropRect": [[crop_area.x, crop_area.y], [crop_area.width, crop_area.height]],
            "videoCodec": video_codec.unwrap_or("h264".into())
        });

        // TODO: Add a timeout of 5s here and return an error if the recording doesn't start

        println!("recorder_options: {}", recorder_options);

        // wait for 1s
        sleep(Duration::from_secs(2)).await;

        // Start recording
        let output = Command::new(APERTURE_BINARY)
            .args(&[
                "record",
                "--process-id",
                &self.process_id,
                &recorder_options.to_string(),
            ])
            .spawn()
            .unwrap();

        self.recorder = Some(output);

        return Ok(());
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

    pub async fn stop_recording(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        self.throw_if_not_started()?;
        if let Some(recorder) = &mut self.recorder {
            recorder.kill()?;
            recorder.wait()?;
        }

        let tmp_path = self
            .tmp_path
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Temporary path not found"))?;

        Ok(tmp_path.to_string_lossy().to_string())
    }

    // TODO: create pause and resume functionality
}
