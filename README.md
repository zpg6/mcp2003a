# mcp2003a

Embedded Rust Microchip MCP2003A/B LIN transceiver driver with `embedded-hal` blocking and async traits for `no-std` environments.

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
> This crate may not be suitable for production use. It was written as hands-on learning exercise of a well-documented specification.
> It may not cover all edge cases or vendor-specific implementations. Please use with caution.

This driver attempts to be a simple reflection of the well-documented instructions from the LIN specification:
https://www.lin-cia.org/fileadmin/microsites/lin-cia.org/resources/documents/LIN_2.2A.pdf

## Alternatives

- [https://github.com/Skuzee/LIN_BUS_lib-Skuzee](https://github.com/Skuzee/LIN_BUS_lib-Skuzee) (Arduino)
- [https://github.com/NaokiS28/LINduino](https://github.com/NaokiS28/LINduino) - (Arduino)
- [https://github.com/Sensirion/lin-bus-rs](https://github.com/Sensirion/lin-bus-rs) - (Rust)
- [https://github.com/fernpedro/Two-node-LIN-cluster-with-Arduino](https://github.com/fernpedro/Two-node-LIN-cluster-with-Arduino) - (Arduino)
  - Includes wiring diagram, walkthrough, and photo of a LIN frame on an oscilloscope
- [https://forum.arduino.cc/t/sending-data-using-lin-cominication/1178509](https://forum.arduino.cc/t/sending-data-using-lin-cominication/1178509) - (Arduino) (forum post with example code)
- [https://github.com/gandrewstone/LIN](https://github.com/gandrewstone/LIN) - (C++)
- [https://github.com/fernpedro/LIN-frame-Header-implementation](https://github.com/fernpedro/LIN-frame-Header-implementation) - (C++)

## Similar Projects

- [https://github.com/matt2005/LIN-1](https://github.com/matt2005/LIN-1) - (C++) Supports LIN on MCP2025
- [https://github.com/macchina/LIN](https://github.com/macchina/LIN) - (C++) (Arduino library to add dual LIN support on SAM3X based boards with a TJA1021/TJA1027 transceiver)

## Supported MCP2003 Part Numbers

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

Full Documentation: [https://docs.rs/mcp2003a/latest/mcp2003a/](https://docs.rs/mcp2003a/latest/mcp2003a/)

## Features

Blocking:

- `embedded-hal = "1.0.0"` - Embedded HAL traits for GPIO, UART, and Delay drivers.
- `embedded-hal-nb = "1.0.0"` - Additional non-blocking traits using `nb` crate underneath.

Async:

- `embedded-hal-async = "1.0.0"` - Async traits for async GPIO, and Delay drivers.
- `embedded-io-async = "0.6.1"` - Async traits for async UART drivers.

## Usage

Add the crate to your `Cargo.toml`:

```
cargo add mcp2003a
```

### Examples

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

If you have async UART, GPIO, and Delay drivers that implement the `embedded-hal-async` traits, you can use the async methods (recommended). For example:

```rust
mcp2003a.send_frame_async(0x01, &[0x02, 0x03], 0x05).await.unwrap();
```

### Full Examples

(More coming soon)

- [ESP-32 via ESP-RS](https://github.com/zpg6/mcp2003a/tree/main/examples/mcp2003a-esp-rs) - Example using the MCP2003A with an ESP-32 microcontroller using `esp-idf-hal` (std).
