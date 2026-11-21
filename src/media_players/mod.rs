use std::{path::PathBuf};
use anyhow::Result;

pub mod mpv;

pub trait MediaPlayer {
    fn display_logo(&mut self, secs: u64) -> Result<()>;
    fn loop_video(&mut self, path: PathBuf) -> Result<()>;
    fn stop_video(&mut self) -> Result<()>;
}

