use std::process::Command;

use anyhow::Result;
use log::{info, trace, warn, error, debug};

mod media_players;
mod consts;
mod loopert_api;

use crate::{consts::*, loopert_api::LoopertApi, media_players::mpv::Mpv};

fn main() -> Result<()> {
    env_logger::init();
    info!("starting server");
    let mut loopert = LoopertApi {
        media_player: Mpv::new()
    };
    loopert.display_logo()?;
    loopert.mount()?;
    let output = Command::new("ls")
        .arg("-la")
        .arg(MOUNT_POINT)
        .output()?;
    trace!("ls: {}", String::from_utf8_lossy(&output.stdout));
    loopert.loop_video()?;
    loopert.unmount()?;

    info!("server shutting down");
    Ok(())
}
