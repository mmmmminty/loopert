use std::{path::PathBuf, process::{Child, Command, Stdio}};
use anyhow::{Ok, Result};
use log::info;

use crate::{consts::{LOGO, LOGO_DURATION}, media_players::MediaPlayer};

impl Mpv {
    pub fn new() -> Self {
        Self { child: None }
    }
}

pub struct Mpv {
    child: Option<Child>
}

impl MediaPlayer for Mpv {
    fn display_logo(&mut self, secs: u64) -> Result<()> {
        let status = Command::new("mpv")
            .args([
                "--vo=gpu",
                "--gpu-context=drm",
                "--drm-mode=preferred",
                "--fs",
                "--no-osc",
                "--no-input-default-bindings",
                "--no-terminal",
                "--no-audio",
            ])
            .arg(format!("--image-display-duration={secs}"))
            .arg(LOGO)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .status()?;

         if !status.success() {
            info!("show_logo: mpv exited with {}", status);
        }
        info!("show_logo: ended");
        Ok(())
    }

    fn loop_video(&mut self, path: PathBuf) -> Result<()> {
        let child = Command::new("mpv")
            .args([
                "--vo=gpu",
                "--gpu-context=drm",
                "--drm-mode=preferred",
                "--hwdec=drm-copy",
                "--fs",
                "--loop-file=inf",
                "--no-osc",
                "--no-input-default-bindings",
                "--no-terminal",
            ])
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit()) // mpv errors show up in logs / journalctl
            .spawn()?;
        info!("play_video: mpv started (pid {})", child.id());
        self.child = Some(child);

        Ok(())
    }

    /// Kills mpv and waits for it to exit, so the files are released before unmounting.
    fn stop_video(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        info!("stop_video: mpv stopped");
        Ok(())
    }
}