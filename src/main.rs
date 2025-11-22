// handle parsing command line args
use clap::{Parser, Subcommand};

// import the syscalls
use nix::sched::{unshare, CloneFlags};
use std::process::{Command};

// change hostname (proof)
use nix::unistd::{sethostname, execvp, getpid, chroot, chdir};
use std::ffi::CString;

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

fn run(cmd: String, args: Vec<String>) {
    println!("Parent: Setting up isolation...");

    // 1. magic syscall: create new "rooms" for Hostname (UTS) and PIDs
    unshare(CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWPID).unwrap();

    // 2. Re-Exec: spawn a copy of OURSELVES into those new rooms
    // we call the hidden "child" command we defiend in step 1
    let mut child = Command::new("/proc/self/exe")
        .arg("child")
        .arg(cmd)
        .args(args)
        .spawn()
        .unwrap();


    child.wait().unwrap(); // if a child prints "I am inside...", we know the process cloning worked
}

fn child(cmd: String, args: Vec<String>) {
    println!("Child (PID: {}): Configuring container...", getpid());

    // 1. Proof: set a hostname
    // if isolation failed, this would rename my actual laptop (bad!)
    // since we unshared UTS, this is safe
    sethostname("carapace-container").unwrap();

    // 2. The Jail: Restrict Filesystem access to the `rootfs` folder
    println!("Child: Entering chroot jail...");

    // change root to 'rootfs' folder
    chroot("rootfs").expect("Failed to chroot. Did you create the 'rootfs' folder?");

    // security best practice: moving working dir to the new root ("/")
    chdir("/").expect("Failed to chdir to /");

    // 3. The Handover: Delete this Rust program from memory and load /bin/sh
    let c_cmd = CString::new(cmd.clone()).unwrap();
    let c_args: Vec<CString> = std::iter::once(cmd) // start with program name
        .chain(args.into_iter()) // add args
        .map(|s| CString::new(s).unwrap()) // convert everything to CString
        .collect();
    // point of no return: once execvp() runs, my Rust code is gone
    execvp(&c_cmd, &c_args).unwrap();
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {cmd, args} => {
            println!("Parent: I need to start a container for '{}'", cmd);
            run(cmd, args);
        }
        Commands::Child {cmd, args} => {
            println!("Child: I am inside the container running '{}'", cmd);
            child(cmd, args);
        }
    }
}