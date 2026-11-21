use std::{fs, path::PathBuf, process::Command};

use anyhow::Result;
use log::{info, trace, warn, error, debug};

use crate::{consts::*, media_players::MediaPlayer};

pub struct LoopertApi<T: MediaPlayer> {
    pub(crate) media_player: T
}

impl<T: MediaPlayer> LoopertApi<T> {
    /// Finds the device at the specified `PORT_TAG` provided by `consts.rs`. Will automatically return the largest partition.
    fn find_device() -> Option<PathBuf> {
        fn partition_size(dev: &PathBuf) -> Option<u64> {
            let real = fs::canonicalize(dev).ok()?;            // /dev/sda2
            let name = real.file_name()?.to_str()?;            // "sda2"
            let s = fs::read_to_string(format!("/sys/class/block/{name}/size")).ok()?;
            trace!("partition_size: size [{s}]");
            s.trim().parse::<u64>().ok()                       // sectors
        }

        debug!("find_device: started");
        let mut parts: Vec<(u64, PathBuf)> = Vec::new();
        let mut whole: Option<PathBuf> = None;

        if let Ok(rd) = fs::read_dir("/dev/disk/by-path") {
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.contains(PORT_TAG) {
                    continue;
                }
                let path = entry.path();
                if name.contains("-part") {
                    if let Some(sz) = partition_size(&path) {
                        parts.push((sz, path));
                    }
                } else {
                    whole = Some(path); // no partition table
                }
            }
        }

        parts.sort_by_key(|(size, _)| *size); // largest last
        trace!("find_device: parts: {:?}", parts);
        let mut out: Vec<PathBuf> = parts.into_iter().map(|(_, p)| p).collect();
        
        debug!("find_device: ended");
        if out.is_empty() {
            whole
        } else {
            out.pop()
        }
    }

    /// Finds the first video file in the mount point specified by `consts.rs`. Returns None if no video files are found.
    fn find_video_path() -> Option<PathBuf> {
        debug!("find_video_path: started");
        let mut videos: Vec<PathBuf> = fs::read_dir(MOUNT_POINT)
            .ok()?
            .flatten()
            .filter(|e| {
                // skip hidden files such as macOS "._video.mp4" junk
                !e.file_name().to_string_lossy().starts_with('.')
            })
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| {
                p.extension()
                    .and_then(|x| x.to_str())
                    .map(|x| EXTS.contains(&x.to_lowercase().as_str()))
                    .unwrap_or(false)
            })
            .collect();

        trace!("find_video_path: videos: {:?}", videos);
        videos.sort(); // alphabetical, just play the first one

        debug!("find_video_path: ended");
        videos.into_iter().next()
    }

    /// Attempts to mount the device specified by `consts.rs`
    pub fn mount(&self) -> Result<()> {
        info!("mount: started");
        let Some(device) = Self::find_device() else {
            error!("mount: No device found");
            return Err(anyhow::anyhow!("mount: No device found"));
        };

        // e.g.
        // mount -o ro,noatime,nodev,nosuid,noexec platform-fd500000.pcie-pci-0000:01:00.0-usbv2-0:1.3:1.0-scsi-0:0:0:0-part1 /media/loopert
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
    pub fn unmount(&self) -> Result<()> {
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

    /// Displays the logo specified by `consts.rs` for the duration specified by `consts.rs`
    pub fn display_logo(&mut self) -> Result<()> {
        info!("display_logo: started");
        self.media_player.display_logo(LOGO_DURATION)?;
        info!("display_logo: ended");
        Ok(())
    }

    /// Loops the video file found by `find_video_path`.
    pub fn loop_video(&mut self) -> Result<()> {
        info!("loop_video: started");
        let video_path = Self::find_video_path().ok_or_else(|| anyhow::anyhow!("loop_video: No video file found"))?;
        info!("loop_video: looping video: {}", video_path.display());

        self.media_player.loop_video(video_path)?;

        std::thread::sleep(std::time::Duration::from_secs(30));
        self.media_player.stop_video()?;

        info!("loop_video: ended");
        Ok(())
    }
}
