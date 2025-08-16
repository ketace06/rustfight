# RustFight Setup

## Windows
1. **Install Rust**  
   Install Rustup: [https://www.rust-lang.org/](https://www.rust-lang.org/)  
   Ensure `rustc` and `cargo` are in your PATH.

2. **Install Build Tools**  
   Install Visual Studio Build Tools with C++ workload: [Download](https://visualstudio.microsoft.com/downloads/)

3. **Clone Project**  
   ```bash
   git clone https://github.com/ketace06/rustfight.git
   ```
4. **Build and run**
   ```bash
   cargo run
   ```
5. **Update**

   Pull changes from GitHub and recompile.

## Linux 
1. **Install Rust**
    Install Rust
    Install Rustup via terminal script and add Rust to PATH.
    https://rustup.rs/
  
    After installation, ensure Rust’s bin directory is in your PATH:  source $HOME/.cargo/env

2. **Install Tools**
    Install build-essential, clang, gcc, and required graphics/audio libraries.

    ```bash
    sudo apt update
    sudo apt install build-essential pkg-config cmake
    ```
    or use dnf instead of apt, if you are running Fedora

    then install a c++ compiler if not included 
    ```bash 
    sudo apt install gcc g++
    ```

    For graphics/audio (required by Bevy):
    ```bash
    sudo apt install libx11-dev libxcursor-dev libxrandr-dev libxinerama-dev \
    libxi-dev libgl1-mesa-dev libasound2-dev libpulse-dev libudev-dev \
    libssl-dev libdrm-devv
    ```

    Optional but recommended: Clang for certain crates:
    ```bash
    sudo apt install clang

    ```

3. **Clone Project**  
   ```bash
    git clone https://github.com/ketace06/rustfight.git
   ```
4. **Build and run**
   ```bash
    cargo run
   ```
5. **Update**

   Pull changes from GitHub and recompile.
