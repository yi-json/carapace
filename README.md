# Carapace

A lightweight container runtime written in Rust.

## Getting Started

### Prerequisites
- Linux (or a Linux VM)
- Rust (cargo)

### Installation
1. Clone the repository:
    ```bash
    git clone https://github.com/yi-json/carapace.git
    cd carapace
    ```
2. Prepare the filesystem: This downloads the minimum Alpine Linux rootfs needed for the container
    ```bash
    ./setup.sh
    ```
3. Run a container:
    ```bash
    sudo cargo run -- run /bin/sh
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

Right now, our container feels safe, but it has a massive security hole.

When we run `ls /home/ubuntu`, we still see our project files and everything on our host machine
* We created "walls" for Process IDs and Hostnames, but we are still looking at the Host's Filesystem
* It's like locking a thief in a room (Namespace) but leaving the window open to the bank vault (file system)

To fix this, we'll use **Alpine Linux** to use as our "jail"

When implmented, we have accomplished the following:
* The **Kernel** is the Ubuntu Kernel from the VM
* The **Userland** (files, shell, tools) is Alpine Linux
* The **Process** is trapped in a jail (`chroot`) and a clean room (`namespaces`)

## Resources
* [Introduction to containers](https://litchipi.github.io/2021/09/20/container-in-rust-part1.html)