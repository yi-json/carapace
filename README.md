# Carapace

A lightweight container runtime written in Rust.

## Getting Started

### Prerequisites
- Linux (or a Linux VM)
- Rust (cargo)

### Installation
0. **Setup**:
    * If you're already using Linux, then feel free to skip (also you're a nerd).
    * Otherwise, we have to create a Virtual Machine, here's how:
        1. Install mutlipass, on brew it's `brew install --cask multipass`
        2. Launch a Development VM - Run this in your terminal to create an Ubuntu 24.04 VM with enough RAM to compile Rust:
            * `multipass launch --name rusty-box --cpus 4 --memory 4G --disk 20G 24.04`
            * `multipass shell rusty-box`
        3. Once inside the VM, run:
            * `sudo apt update && sudo apt install -y build-essential`
            * `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
            * `source "$HOME/.cargo/env"`
1. **Clone the repository (inside your Linux environment):**
    ```bash
    git clone https://github.com/yi-json/carapace.git
    cd carapace
    ```
2. **Prepare the filesystem:** This downloads the minimum Alpine Linux rootfs needed for the container
    ```bash
    ./setup.sh
    ```
3. **Run a container:**
    ```bash
    sudo cargo run -- run /bin/sh
    ```
4. **Explore the features:**
   Once inside the container, try these commands to verify the isolation and resource limits.

   * **Check Process Isolation (PID Namespace):**
     ```bash
     ps
     # You should see PID 1 for /bin/sh.
     ```

   * **Check Filesystem Isolation (Chroot):**
     ```bash
     ls /
     cat /etc/os-release
     # You are trapped inside Alpine Linux.
     ```

   * **Test Resource Limits (Cgroups):**
     Carapace limits the container to **5 processes**. Try to crash it by spawning 6 background jobs:
     ```bash
     for i in $(seq 1 6); do sleep 100 & done
     # Result: "can't fork: Resource temporarily unavailable" (The kernel blocked you!)
     ```


## Documentation
Documenting all of the steps I've done for my future self.

### Phase 0: Setting up the Environment
Step 1: Get a Linux VM
1. Install mutlipass via Homebrew: `brew install --cask multipass`
2. Spin up your customized VM because we need a bit more CPU/RAM than the default for compiling Rust efficiently:
    * Creates an Ubuntu 24.04 VM named 'rusty-box' with 4 CPUs, 4GB RAM, and 20GB disk
    * `multipass launch --name rusty-box --cpus 4 --memory 4G --disk 20G 24.04`
3. Enter the VM: `mutlipass shell rusty-box` - this is how you enter from now on after setting up
    * You are not "inside" Linux - terminal prompt should change to `ubuntu@rusty-box`

Step 2: Set up the Rust Toolchain inside the VM
1. Update and install build tools using `build-essential` (gcc, linker): `sudo apt update && sudo apt install -y build-essential`
2. Install Rust within the container: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. Refresh Path: `source "$HOME/.cargo/env"`

Step 3: Connect VS Code
1. Generate an SSH key on your Mac using `ssh-keygen -t ed25519`
2. Copy public key: `cat ~/.ssh/id_ed25519.pub | pbcopy`
3. Go into VM and add key to authorized_keys: `echo "PASTE_YOUR_KEY_HERE" >> ~/.ssh/authorized_keys`
4. Get VM's IP address using `ip addr show enp0s1`
5. In VSCode, press `Cmd + Shift + P` -> **Remote-SSH: Connect to Host...**
6. Enter `ubuntu@<THE_IP_ADDR_YOU_FOUND>`

Step 4: Provisioning an SSH Key
1. Add your email in the VM terminal: `ssh-keygen -t ed25519 -C "your_email@gmail.com"`
2. Copy key: `cat ~/.ssh/id_ed25519.pub | pbcopy`
3. Go to [GitHub SSH Settings](https://github.com/settings/keys)
4. Click New SSH Key adn enter details
5. Now you are able to git clone the repository

Why do we do this?
1. The Concept: Asymmetric Cryptography
Unlike a password (where you and GitHub both know the secret "12345"), this uses two separate mathematical keys that work as a pair.
    * The Private Key (id_ed25519): This stays inside your VM. You never show this to anyone. Think of this as the physical key in your pocket.
    * The Public Key (id_ed25519.pub): This is what you copied to GitHub. Think of this as the lock cylinder that you install on a door.
2. The "Handshake"
When you run git clone in a moment, this happens:
    * GitHub (Server): "I see you are trying to connect. I have a lock (Public Key) on file for you. I am going to send you a complex math problem that can only be solved by the matching Key."
    Your VM (Client): "No problem. I have the Private Key." It solves the math problem locally and sends the answer back.
GitHub: "The answer is correct. You are authenticated."

### Phase 1: Skeleton Code
1. The Interface ("Manager"; Can I talk to it?)
    * Before worrying about Linux syscalls, I need a program that understands my commands
    * I need it to distinguish between "Me (the user)" and "it (the internal user)"
    * **Goal**: Make `cargo run --run /bin/sh` print a message
```{rust}
use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {cmd, args} => {
            println!("Parent: I need to start a container for '{}'", cmd);
            // TODO: Create isolation here
        }
        Commands::Child {cmd, args} => {
            println!("Child: I am inside the container running '{}'", cmd);
            // TODO: Become the shell here
        }
    }
}
```

2. The Architect (Building the Walls)
    * Now that the interface works, we need the Parent to actually create the isolation. We do this by successfully launching a second process that is "disconnected" from the host
    * **Goal**: When I run the code, I want to see the Parent print "Setting up..." and then the Child print "I am inside..."
```{rust}
use nix::sched::{unshare, CloneFlags};
use std::process::{Command, Stdio};

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
```

3. The Tenant (Moving In)
    * Now I am inside the container (the `child` block runs)
    * I need to prove I'm isolated and then actually become the shell
    * **Goal**: Change the hostname (proof) and swap a Rust process for a Shell process
    * How to test: Build as User, Run as Root (compile the code as yourself), but execute the final binary as root
        * `cargo build`
        * `sudo ./target/debug/carapace run /bin/sh`
    * Successful Check:
        1. You will see your "Parent" and "Child" print statements.
        2. You will be in a new shell prompt denoted as `#`
            * This means you are running `bin/sh` as the root user inside your new isolated environment
        3. Type hostname — it should say carapace-container.
            * Proves the UTS Namespace worked
            * We renamed the machine inside the container, but your actual VM is still called `rusty-box`
        4. Type exit to leave.

### Phase 2: Improving the Security by Adding Jail
Right now, our container feels safe, but it has a massive security hole.

When we run `ls /home/ubuntu`, we still see our project files and everything on our host machine
* We created "walls" for Process IDs and Hostnames, but we are still looking at the Host's Filesystem
* It's like locking a thief in a room (Namespace) but leaving the window open to the bank vault (file system)

To fix this, we'll use **Alpine Linux** to use as our "jail"

When implmented, we have accomplished the following:
* The **Kernel** is the Ubuntu Kernel from the VM
* The **Userland** (files, shell, tools) is Alpine Linux
* The **Process** is trapped in a jail (`chroot`) and a clean room (`namespaces`)

Example usage:
```bash
ubuntu@rusty-box:~/github/carapace$ sudo ./target/debug/carapace run /bin/sh
Parent: I need to start a container for '/bin/sh'
Parent: Setting up isolation...
Child: I am inside the container running '/bin/sh'
Child (PID: 1): Configuring container...
Child: Entering chroot jail...
/ # ls
bin     dev     etc     home    lib     media   mnt     opt     proc    root    rootfs  run     sbin    srv     sys     tmp     usr     var
/ # ls /home
/ # cat /etc/os-release
NAME="Alpine Linux"
ID=alpine
VERSION_ID=3.18.4
PRETTY_NAME="Alpine Linux v3.18"
HOME_URL="https://alpinelinux.org/"
BUG_REPORT_URL="https://gitlab.alpinelinux.org/alpine/aports/-/issues"
/ # 
```

To achieve this, we modify the `child()` function to perform a "FileSystem Swap" before executing the user's command. 

We use two specific system calls:
    1. `chroot("rootfs")`: This changes the Root Directory for the current process. The Kernel now translates / to mean the ./rootfs folder we downloaded.
    2. `chdir("/")`: This is a critical security step. Even after changing the root, the process might technically be "standing" in a directory outside the jail. Changing the directory to / ensures we are physically inside the new environment.

```rust
use nix::unistd::{chroot, chdir};

fn child(cmd: String, args: Vec<String>) -> Result<()> {
    // ... previous namespace code ...

    println!("Child: Entering chroot jail...");
    
    // 1. The Lock: Restrict filesystem access to the 'rootfs' folder
    chroot("rootfs")?; 
    
    // 2. The Entry: Move current working directory into the new root
    chdir("/")?;

    // ... proceed to execvp ...
    Ok(()) // this is needed because of specified return type Result<()>
}
```

### Phase 3: Adding Control Groups (`Cgroups`)
Now, we are moving from a "process isolation trick" to a real "container runtime"  that can enforce limits
    * This is similar to to the resource management Google cares about for Borg/Kubernetes
1. Cgroup Logic
    * We add `setup_cgroups()` to set up the Cgroup hierarchy and limits
    * We add `clean_cgroups()` to clean the Cgroup directory after we are done
2. Update Run Logic
    * We need to turn on the Cgroup on **before** we start the container, and turn if off **after** the container dies

#### Comparing `run()` before and after:
**Before (Prototype)**:
```rust
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
```

**After (Production Ready)**:
```rust
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
```
### Breakdown

1. **Error Handling: `unwrap()` vs `?`**
    - **Old**: If `unshare` fails (e.g., forgetting `sudo`), the program **panics**.
        - It prints a scary error message and instantly crashes.
        - The cleanup code (deleting cgroups) never runs.
    - **New**: Uses `?` and returns `Result<()>`.
        - If `unshare` fails, the error is passed up to `main()`.
        - This allows us to handle it gracefully.
        - You never crash on purpose — you propagate errors so you can log them or clean up resources before exiting.

2. **Resource Lifecycle**
    - We wrapped the child process in a “Setup” and “Teardown” sandwich.
    - **Setup** using `setup_cgroups` — done **before** the child starts.
        - Why? If we start the child first, it might run away and spawn 1,000 processes before we can apply the limit.
        - We build the “cage” (Cgroup) first, then put the process inside.
    - **Cleanup** using `clean_cgroups` — done **after** `child.wait()`.
        - Why? In Linux, cgroups are directories in the kernel’s memory (`/sys/fs/cgroup/...`).
        - If your program finishes and doesn’t delete that folder, it stays there forever until you reboot.
        - This is a **memory leak**, so we clean it to ensure we leave the system tidy.


#### Comparing `child()` before and after:
**Before (Prototype)**:
```rust
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
```

**After (Production Ready)**:
```rust
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
    let c_cmd = CString::new(cmd.clone()).unwrap();
    let c_args: Vec<CString> = std::iter::once(cmd) // start with program name
        .chain(args.into_iter()) // add args
        .map(|s| CString::new(s).unwrap()) // convert everything to CString
        .collect();
    // point of no return: once execvp() runs, my Rust code is gone
    execvp(&c_cmd, &c_args)?;

    Ok(())
}
```

### Breakdown

1. **In New – Deep Dive: What is _Mounting_?**
    - In Linux, *“everything is a file”* — hard drive, keyboard input, list of running processes.
    - The `ps` command (and `top`, `htop`) doesn't “talk to the kernel.” It simply reads files inside the directory `/proc`.
    - If that directory is empty, `ps` thinks no processes exist.
    - **Problem**: When we did `chroot("rootfs")`, we trapped the process inside the Alpine Linux folder. Inside that folder, there is a directory called `proc`, but **it is empty** — just a regular folder on our hard drive.
    - **Solution (Mounting)**: We need to map the kernel’s internal memory (where it keeps track of PIDs) onto that empty folder — a **pseudo-filesystem**.

      ```rust
      mount(
          Some("proc"),      // 1. Source (The Label)
          "/proc",           // 2. Target (Where to put it)
          Some("proc"),      // 3. Filesystem Type (The Mechanism)
          MsFlags::empty(),  // 4. Flags (Read-Write, etc.)
          None::<&str>       // 5. Data (Options)
      )?;
      ```


When implemented, we have accomplished the following -> run `sudo ./target/debug/carapace run /bin/sh`:
    * Run `ps` - You should see PIDs.
    * Run `mount` - You should see proc on /proc type proc.

### Phase 4: Foreign Function Interface (FFI)
We will write a small C++ function that reads the Kernel Version (using uname) and prints a "Container Ready" banner. We will compile this C++ code and call it from inside your Rust runtime.

1. To do this, we update `Cargo.toml` to include the build dependency `cc`
2. Create a new file `src/inspector.cpp`
3. Write a build script `build.rs` at the root level to compile the C++ file before building Rust
4. Call the external function inside the `run` loop


## Resources
* [Introduction to containers](https://litchipi.github.io/2021/09/20/container-in-rust-part1.html)
* [Linux Man Pages: Namespaces](https://man7.org/linux/man-pages/man7/namespaces.7.html)
* [Linux Man Pages: Cgroups](https://man7.org/linux/man-pages/man7/cgroups.7.html)