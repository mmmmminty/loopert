use std::{fs, path::PathBuf, process::Command, time::Duration};

use anyhow::Result;
use log::{info, trace, warn, error};

mod consts;
use consts::*;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting server");
    let loopert = LoopertApi {};

    loopert.mount()?;
    std::thread::sleep(Duration::from_secs(1));
    loopert.unmount()?;

    info!("Server shutting down");
    Ok(())
}

struct LoopertApi {
    
}

impl LoopertApi {
    /// Finds the device at the specified `PORT_TAG` provided by `consts.rs`.
    fn find_device() -> Option<PathBuf> {
        for entry in fs::read_dir("/dev/disk/by-path").ok()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(PORT_TAG) && name.ends_with("-part1") {
                return Some(entry.path()); // full path, e.g. /dev/disk/by-path/...-part1
            }
        }
        None
    }

    /// Attempts to mount the device specified by `consts.rs`
    fn mount(&self) -> Result<()> {
        info!("mount: started");
        let Some(device) = Self::find_device() else {
            error!("mount: No device found");
            return Err(anyhow::anyhow!("mount: No device found"));
        };

        let output = Command::new("mount")
            .args(["-o", "ro,noatime,nodev,nosuid,noexec"])
            .arg(&device)
            .arg(MOUNT_POINT)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("mount: FAILED {}", String::from_utf8_lossy(&output.stderr));
        }

        trace!("mount: success");
        info!("mount: ended");
        Ok(())
    }

    /// Attempts to unmount the device specified by `consts.rs`
    fn unmount(&self) -> Result<()> {
        info!("unmount: started");

        let output = Command::new("umount")
            .arg("-l")
            .arg(MOUNT_POINT)
            .output()?;

        if !output.status.success() {
            anyhow::bail!("umount: FAILED {}", String::from_utf8_lossy(&output.stderr));
        }

        trace!("unmount: success");
        info!("unmount: ended");
        Ok(())
    }
}
