// handle parsing command line args
use clap::{Parser, Subcommand};

// import the syscalls
use nix::sched::{unshare, CloneFlags};
use std::process::{Command, Stdio};

// change hostname (proof)
use nix::unistd::{sethostname, execvp, getpid, chroot, chdir};
use std::ffi::CString;

// add filesystem operations for cgroups
use anyhow::Result;
use std::fs;
use std::path::Path;
use nix::mount::{mount, MsFlags};
const CGROUP_NAME: &str = "carapace-container";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { cmd: String, args: Vec<String> }, // User types this
    Child { cmd: String, args: Vec<String> } // Internal use only
}

// Sets up the Cgroup hierarchy and limits
fn setup_cgroups() -> Result<()> {
    println!("Parent: Setting up Cgroups...");

    // 1. define the location: /sys/fs/cgroup
    let cgroup_path = Path::new("/sys/fs/cgroup").join(CGROUP_NAME);

    // 2. create the dir
    // in CGroups v2, creating a dir auto creates the control files inside it
    if !cgroup_path.exists() {
        fs::create_dir(&cgroup_path)?;
    }

    // 3. set the limit: max 5 processes
    // we write "5" into the 'pids.max' file
    let pids_max = cgroup_path.join("pids.max");
    fs::write(pids_max, "5")?;

    // 4. add ourselves (the parent) to this Cgroup
    // the child process we spawn later will inherit this Cgroup automatically
    let procs = cgroup_path.join("cgroup.procs");
    let pid = getpid();
    fs::write(procs, pid.to_string())?;

    Ok(())
}

fn clean_cgroups() -> Result<()> {
    println!("Parent: Cleaning up Cgroups...");
    let cgroup_path = Path::new("/sys/fs/cgroup").join(CGROUP_NAME);
    if cgroup_path.exists() {
        fs::remove_dir(&cgroup_path)?;
    }
    Ok(())
}

fn run(cmd: String, args: Vec<String>) -> Result<()> {
    // setup cgroups (limit resources)
    setup_cgroups()?;

    println!("Parent (PID: {}): Setting up isolation...", getpid());

    // unshare namespaces (UTS, PID, and Mount)
    let flags = CloneFlags::CLONE_NEWUTS 
              | CloneFlags::CLONE_NEWPID 
              | CloneFlags::CLONE_NEWNS;
    unshare(flags)?;

    // 2. Re-Exec: spawn a copy of OURSELVES into those new rooms
    // we call the hidden "child" command we defiend in step 1
    let mut child = Command::new("/proc/self/exe")
        .arg("child")
        .arg(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;


    child.wait()?; // if a child prints "I am inside...", we know the process cloning worked
    
    // cleanup
    clean_cgroups()?;

    Ok(())
}

fn child(cmd: String, args: Vec<String>) -> Result<()> {
    println!("Child (PID: {}): Entering container...", getpid());

    // 1. Hostname
    sethostname("carapace-container")?;

    // 2. The Jail: Restrict Filesystem access to the `rootfs` folder
    println!("Child: Entering chroot jail...");
    chroot("rootfs")?; // change root to 'rootfs' folder
    chdir("/")?;  // security best practice: moving working dir to the new root ("/")

    // 3. mount /proc (added this back so `ps` works!)
    println!("Child: Mounting /proc...");
    mount(
        Some("proc"),
        "/proc",
        Some("proc"),
        MsFlags::empty(),
        None::<&str>
    )?;

    // 4. execute user command
    let c_cmd = CString::new(cmd.clone())?;
    let c_args: Vec<CString> = std::iter::once(cmd) // start with program name
        .chain(args.into_iter()) // add args
        .map(|s| CString::new(s).unwrap()) // convert everything to CString
        .collect();
    // point of no return: once execvp() runs, my Rust code is gone
    execvp(&c_cmd, &c_args)?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {cmd, args} => run(cmd, args),
        Commands::Child {cmd, args} => child(cmd, args),
    }
}