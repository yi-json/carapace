// This contains the heavy lifting: Namespaces, Mounts, and the Parent/Child process logic.

use anyhow::Result;
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{execvp, sethostname, getpid, chroot, chdir};
use nix::mount::{mount, MsFlags};
use std::ffi::CString;
use std::process::{Command, Stdio};
use crate::cgroups; // Access the cgroups module we just made

// --- PARENT ---
pub fn run(cmd: String, args: Vec<String>) -> Result<()> {
    cgroups::setup()?;

    println!("Parent (PID: {}): Setting up isolation...", getpid());

    let flags = CloneFlags::CLONE_NEWUTS 
              | CloneFlags::CLONE_NEWPID 
              | CloneFlags::CLONE_NEWNS; 
    unshare(flags)?;

    let mut child_proc = Command::new("/proc/self/exe")
        .arg("child")
        .arg(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    child_proc.wait()?;
    cgroups::clean()?;
    Ok(())
}

// --- CHILD ---
pub fn child(cmd: String, args: Vec<String>) -> Result<()> {
    println!("Child (PID: {}): Entering container...", getpid());

    sethostname("carapace-container")?;

    println!("Child: Entering chroot jail...");
    chroot("rootfs")?;
    chdir("/")?;

    println!("Child: Mounting /proc...");
    mount(
        Some("proc"),
        "/proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )?;

    let c_cmd = CString::new(cmd.clone())?;
    let c_args: Vec<CString> = std::iter::once(cmd)
        .chain(args.into_iter())
        .map(|s| CString::new(s).unwrap())
        .collect();
    
    execvp(&c_cmd, &c_args)?;
    Ok(())
}