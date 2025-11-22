### Documentation
Documenting all of the steps I've done for my future self.

#### Phase 0: Setting up the Environment
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


### Resources
* [Introduction to containers](https://litchipi.github.io/2021/09/20/container-in-rust-part1.html)