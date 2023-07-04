// Inside Aperture Impl
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

async fn pause(self) {
    self.throw_if_not_started();
    self.send_event("pause", None).await;
}

async fn resume(self) {
    self.throw_if_not_started();
    self.send_event("resume", None).await;
}

async fn isPaused(self) -> Result<bool, Box<dyn std::error::Error>> {
    self.throw_if_not_started();
    let value = self
        .send_event("isPaused", Some(|value| value == "true"))
        .await
        .unwrap_or(false); // Default to false if the event value is not available
    Ok(value)
}
