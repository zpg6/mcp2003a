# mcp2003a

Rust crate for basic `no_std` LIN Bus communications with MCP2003A LIN Transceiver. Uses `embedded-hal` digital traits for GPIO and `embedded-hal-nb` Serial traits for UART.

- `embedded-hal = "1.0.0"` - Major breaking changes versus 0.2.x implementations.
- `embedded-hal-nb = "1.0.0"` - Additional non-blocking traits using `nb` crate underneath.

**⚠️ WORK IN PROGRESS**

Full Documentation: [https://docs.rs/mcp2003a/latest/mcp2003a/](https://docs.rs/mcp2003a/latest/mcp2003a/)

## References

- [MCP2003A Product Page](https://www.microchip.com/wwwproducts/en/MCP2003A)
- [MCP2003A Datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/DataSheets/20002230G.pdf)

## Usage

```
cargo add mcp2003a
```

### Tested Examples

- [ESP-32 via ESP-RS](https://github.com/zpg6/mcp2003a/tree/main/examples/mcp2003a-esp-rs) - Example using the MCP2003A with an ESP-32 microcontroller using the ESP-RS HAL.

### Pseudo-Code Example

Here is an pseudo-code example of how to use to send and receive LIN frames with the MCP2003A LIN Transceiver:

```rust
use mcp2003a::{
    config::{
        LinBreakDuration, LinBusConfig, LinBusSpeed, LinInterFrameSpace,
        LinReadDeviceResponseTimeout, LinWakeupDuration,
    },
    Mcp2003a, Mcp2003aError,
};

let uart = // Your embedded-hal-nb UART driver (usually same baudrate as LIN Bus)
let break_pin = // Your embedded-hal GPIO output pin driver
let delay_ns = // Your embedded-hal delay driver

// Configure the LIN Bus with the following parameters:
let lin_bus_config = LinBusConfig {
    speed: LinBusSpeed::Baud19200,
    break_duration: LinBreakDuration::Minimum13BitsPlus(1), // Test for your application
    wakeup_duration: LinWakeupDuration::Minimum250Microseconds, // Test for your application
    read_device_response_timeout: LinReadDeviceResponseTimeout::DelayMilliseconds(5), // Test for your application
    inter_frame_space: LinInterFrameSpace::DelayMilliseconds(2), // Test for your application
};

// Now control the MCP2003A LIN Transceiver with LIN configuration and driver
let mut mcp2003a = Mcp2003a::new(uart, break_pin, delay_ns, lin_bus_config);

// Wakeup the LIN Bus
mcp2003a.send_wakeup();

// Send a frame on the LIN bus to a device with Command frame of 0x00:
// - Id: 0x00
// - Data: [0x02, 0x03]
// - Checksum: 0x05
match mcp2003a.send_frame(0x00, &[0x02, 0x03], 0x05) {
   Ok(frame) => {
        // Frame sent
        log::info!("Sent data to LIN Id 0x00: {:?}", frame);
    }
    Err(e) => {
        // Error sending the frame
        log::error!("Error sending frame 0x00: {:?}", e);
    }
}

// Read the feedback / diagnostic frame with Id 0x01:
// - Id: 0x01
// - Data: We provide an 8-byte buffer to store the data
let mut data = [0u8; 8];
match mcp2003a.read_frame(0x01, &mut data) {
    Ok(len) => {
        // Data is stored in the buffer
        log::info!("Received data from LIN Id 0x01: {:?}", &data[..len]);
    }
    Err(e) => {
        // Error reading the frame
        match e {
            Mcp2003aError::LinDeviceNoResponse => {
                log::warn!("No response from frame 0x01... this device may be offline.");
            }
            _ => {
                log::error!("Error reading frame 0x01: {:?}", e);
            }
        }
    }
}
```
