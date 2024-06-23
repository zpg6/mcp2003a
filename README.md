# mcp2003a

This Rust crate provides basic `no_std` LIN Bus communications with MCP2003A LIN Transceiver. Uses `embedded-hal` and `embedded-io` traits.

⚠️ WORK IN PROGRESS

## Usage

```rust
use mcp2003a::{
    LinBusConfig, 
    LinBusSpeed,
    LinBreakDuration,
    LinReadDeviceResponseTimeout,
    LinInterFrameSpace,
    Mcp2003a,
};

let uart = // Your embedded-hal UART driver
let break_pin = // Your embedded-hal GPIO output pin driver
let delay_ns = // Your embedded-hal delay driver

let lin_bus_config = LinBusConfig {
    speed: LinBusSpeed::Baud19200,
    break_duration: LinBreakDuration::Minimum13Bits,
    read_device_response_timeout: LinReadDeviceResponseTimeout::DelayMilliseconds(1),
    inter_frame_space: LinInterFrameSpace::DelayMilliseconds(1),
};

let mut mcp2003a = Mcp2003a::new(uart, break_pin, delay_ns, lin_bus_config);

// Read the feedback / diagnostic frame with Id 0x01:
// - Id: 0x01
// - Data: We provide an 8-byte buffer to store the data
let mut data = [0u8; 8];
match mcp2003a.read_frame(0x01, &mut data) {
    Ok(len) => {
        if len > 0 {
            // Data is stored in the buffer
        } else {
            // No data received
        }
    },
    Err(_) => {
        // Error reading the frame
    }
}

// Send a frame on the LIN bus to a device with Command frame of 0x00:
// - Id: 0x00
// - Data: [0x02, 0x03]
// - Checksum: 0x04
match mcp2003a.send_frame(0x00, &[0x02, 0x03], 0x04) {
    Ok(_) => {
        // Frame sent
    },
    Err(_) => {
        // Error sending the frame
    }
}
```

## References

- [MCP2003A Product Page](https://www.microchip.com/wwwproducts/en/MCP2003A)
- [MCP2003A Datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/DataSheets/20002230G.pdf)
