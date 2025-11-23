// This file handles the Linux Cgroup logic. It’s a self-contained module.

use anyhow::Result;
use std::fs;
use std::path::Path;
use nix::unistd::getpid;

const CGROUP_NAME: &str = "carapace-container";

pub fn setup() -> Result<()> {
    println!("Parent: Setting up Cgroups...");
    let cgroup_path = Path::new("/sys/fs/cgroup").join(CGROUP_NAME);

    if !cgroup_path.exists() {
        fs::create_dir(&cgroup_path)?;
    }

    // Limit to 5 processes
    let pids_max = cgroup_path.join("pids.max");
    fs::write(pids_max, "5")?;

    // Add parent to Cgroup
    let procs = cgroup_path.join("cgroup.procs");
    let pid = getpid();
    fs::write(procs, pid.to_string())?;

    Ok(())
}

pub fn clean() -> Result<()> {
    println!("Parent: Cleaning up Cgroups...");
    let cgroup_path = Path::new("/sys/fs/cgroup").join(CGROUP_NAME);
    if cgroup_path.exists() {
        fs::remove_dir(&cgroup_path)?;
    }
    Ok(())
}