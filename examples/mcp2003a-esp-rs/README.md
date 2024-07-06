# mcp2003a-esp-rs

This example demonstrates using the `mcp2003a` crate with an ESP32.

Since the `embedded-hal` and `embedded-hal-nb` traits are implemented via the ESP-RS project (crate: [`esp-idf-hal`](https://crates.io/crates/esp-idf-hal)), we can configure several ESP32 models to use the `mcp2003a` crate.

# Pinout

- 3.3V - VCC
- GND - GND
- GPIO15 - BREAK
- GPIO16 - RX
- GPIO17 - TX

Verified with ESP32 DevKit V1:

![ESP32 Diagram](https://www.circuitstate.com/wp-content/uploads/2022/12/ESP32-DevKit-V1-Pinout-Diagram-r0.1-CIRCUITSTATE-Electronics-2.png)

# Prerequisites

Linux/Mac users: Make sure you have the dependencies installed from are mentioned in the [esp-idf install guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/linux-macos-setup.html#step-1-install-prerequisites). You **don't** need to manually install esp-idf, just its dependencies.

For detailed instructions see [Setting Up a Development Environment](https://esp-rs.github.io/book/installation/index.html) chapter of The Rust on ESP Book.

### Install Rust (with `rustup`)

If you don't have `rustup` installed yet, follow the instructions on the [rustup.rs site](https://rustup.rs)

### Install Cargo Sub-Commands

Run these one by one:

```sh
cargo install cargo-generate
cargo install ldproxy
cargo install espup
cargo install espflash
cargo install cargo-espflash
```

For Linux users:

```
# Debian/Ubuntu/etc.
apt-get install libudev-dev

# Fedora
dnf install systemd-devel
```

> [!NOTE]
> If you are running Linux then `libudev` must also be installed for `espflash` and `cargo-espflash`; this is available via most popular package managers. If you are running Windows you can ignore this step.

### Install Rust & Clang toolchains for Espressif SoCs (with `espup`)

```sh
espup install
# Mac/Linux run:
. $HOME/export-esp.sh
```

> [!WARNING]
> Make sure you source the generated export file, as shown above, in every terminal before building any application as it contains the required environment variables.

# Running the example

```
cargo run --release
```

This will compile the example using the relative path to the `mcp2003a` crate and flash the ESP32 with the compiled binary. When prompted which port to flash to, select either.

If stuck on the 'Connecting...' message, press the EN button on the ESP32 for 1sec to reset it and it should then flash.
