# mcp2003a

Embedded Rust Microchip MCP2003A/B LIN transceiver driver with embedded-hal traits for `no-std` environments.

<a href="https://crates.io/crates/mcp2003a">
    <img src="https://img.shields.io/crates/v/mcp2003a.svg" alt="Crates.io">
</a>
<a href="https://docs.rs/mcp2003a">
    <img src="https://docs.rs/mcp2003a/badge.svg" alt="Documentation">
</a>
<a href="https://github.com/zpg6/mcp2003a">
    <img src="https://img.shields.io/badge/github-zpg6/mcp2003a-black" alt="GitHub Repo">
</a>
<br><br>

> [!WARNING]
> This crate is still in development and may not be suitable for production use.

Full Documentation: [https://docs.rs/mcp2003a/latest/mcp2003a/](https://docs.rs/mcp2003a/latest/mcp2003a/)

## Part Numbers

Tested on:

- [MCP2003A](https://www.microchip.com/wwwproducts/en/MCP2003A) (No Longer Recommended for New Designs)
- MCP2003E

Should also work with:

- [MCP2003B](https://www.microchip.com/en-us/product/MCP2003B) (functional drop-in replacement for MCP2003A)

## References

- [MCP2003A Product Page](https://www.microchip.com/wwwproducts/en/MCP2003A)
- [MCP2003/4/3A/4A Datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/DataSheets/20002230G.pdf)
- [MCP2003A to MCP2003B Migration Guide](https://ww1.microchip.com/downloads/en/DeviceDoc/90003150A.pdf)
- [MCP2003B Datasheet](https://ww1.microchip.com/downloads/en/DeviceDoc/2000546C3.pdf)

## Features

Uses `embedded-hal` digital traits for GPIO and `embedded-hal-nb` Serial traits for UART.

- `embedded-hal = "1.0.0"` - Major breaking changes versus 0.2.x implementations.
- `embedded-hal-nb = "1.0.0"` - Additional non-blocking traits using `nb` crate underneath.

## Usage

Add the crate to your `Cargo.toml`:

```
cargo add mcp2003a
```

### Example

```rust
let mut mcp2003a = Mcp2003a::new(uart2_driver, break_pin_driver, delay);

let lin_bus_config = LinBusConfig {
   speed: LinBusSpeed::Baud19200,
   break_duration: LinBreakDuration::Minimum13Bits, // Test for your application
   wakeup_duration: LinWakeupDuration::Minimum250Microseconds, // Test for your application
   read_device_response_timeout: LinReadDeviceResponseTimeout::DelayMilliseconds(15), // Test for your application
   inter_frame_space: LinInterFrameSpace::DelayMilliseconds(1), // Test for your application
};
mcp2003a.init(lin_bus_config);

mcp2003a.send_wakeup();

// Works for different LIN versions, you calculate id and checksum based on your application
mcp2003a.send_frame(0x01, &[0x02, 0x03], 0x05).unwrap();

let mut read_buffer = [0u8; 8]; // Initialize the buffer to the frame's known size
let checksum = mcp2003a.read_frame(0xC1, &mut read_buffer).unwrap();
```

### Full Examples

(More coming soon)

- [ESP-32 via ESP-RS](https://github.com/zpg6/mcp2003a/tree/main/examples/mcp2003a-esp-rs) - Example using the MCP2003A with an ESP-32 microcontroller using the ESP-RS HAL.
