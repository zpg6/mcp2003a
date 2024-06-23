//! MCP2003A LIN Transceiver Library
//!
//! ⚠️ WORK IN PROGRESS
//!
//! This library provides an `embedded-hal` abstraction for the MCP2003A LIN transceiver using UART
//! and a GPIO output pin for the break signal.
//!
//! LIN (Local Interconnect Network) is a serial network protocol used in automotive and industrial applications.
//! Most automobiles on the road today have several LIN bus networks for various systems like door locks, windows, and more.
//!
//! # MCP2003A
//!
//! The MCP2003A is a LIN transceiver that provides a physical interface between a LIN master and the LIN bus.
//! As such, this code is intended to be used on a LIN master device that communicates with LIN slave devices.
//!
//! See more:
//! - [MCP2003A Product Page](https://www.microchip.com/wwwproducts/en/MCP2003A)
//! - [MCP2003A Datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/OTH/ProductDocuments/DataSheets/20002230G.pdf)
//!
//! # Example
//!
//! ```no_run
//! use mcp2003a::{
//!     LinBusConfig,
//!     LinBusSpeed,
//!     LinBreakDuration,
//!     LinReadDeviceResponseTimeout,
//!     LinInterFrameSpace,
//!     Mcp2003a,
//! };
//!
//! let uart = // Your embedded-hal UART driver
//! let break_pin = // Your embedded-hal GPIO output pin driver
//! let delay_ns = // Your embedded-hal delay driver
//!
//! let lin_bus_config = LinBusConfig {
//!     speed: LinBusSpeed::Baud19200,
//!     break_duration: LinBreakDuration::Minimum13Bits,
//!     read_device_response_timeout: LinReadDeviceResponseTimeout::DelayMilliseconds(1),
//!     inter_frame_space: LinInterFrameSpace::DelayMilliseconds(1),
//! };
//!
//! let mut mcp2003a = Mcp2003a::new(uart, break_pin, delay_ns, lin_bus_config);
//!
//! // Read the feedback / diagnostic frame with Id 0x01:
//! // - Id: 0x01
//! // - Data: We provide an 8-byte buffer to store the data
//! let mut data = [0u8; 8];
//! match mcp2003a.read_frame(0x01, &mut data) {
//!     Ok(len) => {
//!         if len > 0 {
//!             // Data is stored in the buffer
//!         } else {
//!             // No data received
//!         }
//!     },
//!     Err(_) => {
//!         // Error reading the frame
//!     }
//! }
//!
//! // Send a frame on the LIN bus to a device with Command frame of 0x00:
//! // - Id: 0x00
//! // - Data: [0x02, 0x03]
//! // - Checksum: 0x04
//! match mcp2003a.send_frame(0x00, &[0x02, 0x03], 0x04) {
//!     Ok(_) => {
//!         // Frame sent
//!     },
//!     Err(_) => {
//!         // Error sending the frame
//!     }
//! }

#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_io::{Read as UartRead, Write as UartWrite};

/// LIN Break Duration for the MCP2003A transceiver.
/// The specification requires a minimum of 13 bits for the break signal, but the actual underlying
/// implementation of the LIN devices may require more bits for stability (maybe 13 bits + 1 or 2 bits).
#[derive(Clone, Copy, Debug)]
pub enum LinBreakDuration {
    Minimum13Bits,
    Minimum13BitsPlus(u8),
}

impl LinBreakDuration {
    /// Get the duration in nanoseconds for the LIN break duration.
    pub fn get_duration_ns(&self, bit_period_ns: u32) -> u32 {
        match self {
            LinBreakDuration::Minimum13Bits => bit_period_ns * 13,
            LinBreakDuration::Minimum13BitsPlus(bits) => bit_period_ns * ((13u8) + bits) as u32,
        }
    }
}

/// How long to wait after sending a read header before reading the response, allowing the slave device to respond.
/// Typically this is a 1-10 ms delay but can vary by system.
#[derive(Clone, Copy, Debug)]
pub enum LinReadDeviceResponseTimeout {
    None,
    DelayMicroseconds(u32),
    DelayMilliseconds(u32),
}

/// How long to wait after sending a frame before sending the next frame.
/// This applies to both sending and receiving frames. Typically, this is 1-2 ms.
#[derive(Clone, Copy, Debug)]
pub enum LinInterFrameSpace {
    None,
    DelayMicroseconds(u32),
    DelayMilliseconds(u32),
}

impl Default for LinReadDeviceResponseTimeout {
    fn default() -> Self {
        LinReadDeviceResponseTimeout::DelayMilliseconds(1)
    }
}

/// LIN Bus Speeds available for the MCP2003A transceiver in bits per second.
#[derive(Clone, Copy, Debug)]
pub enum LinBusSpeed {
    Baud9600,
    Baud10400,
    Baud19200,
    Baud20000,
}

impl LinBusSpeed {
    /// Get the baud rate in bits per second for the LIN bus speed, just bridging enum to u32.
    pub fn get_baud_rate(&self) -> u32 {
        match self {
            LinBusSpeed::Baud9600 => 9600,
            LinBusSpeed::Baud10400 => 10400,
            LinBusSpeed::Baud19200 => 19200,
            LinBusSpeed::Baud20000 => 20000,
        }
    }

    /// Get the bit period in nanoseconds for the LIN bus speed. This is the time it takes to send one bit.
    pub fn get_bit_period_ns(&self) -> u32 {
        1_000_000_000 / self.get_baud_rate()
    }
}

/// Configuration for the LIN bus.
#[derive(Clone, Copy, Debug)]
pub struct LinBusConfig {
    pub speed: LinBusSpeed,
    pub break_duration: LinBreakDuration,
    pub read_device_response_timeout: LinReadDeviceResponseTimeout,
    pub inter_frame_space: LinInterFrameSpace,
}

/// MCP2003A LIN Transceiver
pub struct Mcp2003a<UART, GPIO, DELAY> {
    uart: UART,
    break_pin: GPIO,
    delay: DELAY,
    config: LinBusConfig,
}

impl<UART, GPIO, DELAY, E> Mcp2003a<UART, GPIO, DELAY>
where
    UART: UartRead<Error = E> + UartWrite<Error = E>,
    GPIO: OutputPin,
    DELAY: DelayNs,
{
    /// Create a new MCP2003A transceiver instance.
    ///
    /// # Arguments
    ///
    /// * `uart` - UART interface for data communication to and from the transceiver.
    /// * `break_pin` - GPIO pin for the break signal.
    /// * `delay` - Delay implementation for break signal timing.
    /// * `config` - Configuration for the LIN bus speed and break duration.
    pub fn new(uart: UART, break_pin: GPIO, delay: DELAY, config: LinBusConfig) -> Self {
        Mcp2003a {
            uart,
            break_pin,
            delay,
            config,
        }
    }

    /// Send a break signal on the LIN bus, pausing execution for at least 730 microseconds (13 bits).
    fn send_break(&mut self) -> Result<(), E> {
        // Calculate the duration of the break signal
        let bit_period_ns = self.config.speed.get_bit_period_ns();
        let break_duration_ns = self.config.break_duration.get_duration_ns(bit_period_ns);

        // Set the break pin low to signal a break
        self.break_pin.set_low().ok();

        // Delay
        self.delay.delay_ns(break_duration_ns);

        // Set the break pin high to end the break
        self.break_pin.set_high().ok();
        Ok(())
    }

    /// Send a frame on the LIN bus with the given ID, data, and checksum.
    pub fn send_frame(&mut self, id: u8, data: &[u8], checksum: u8) -> Result<(), E> {
        // Calculate the frame
        let mut frame = [0u8; 10];

        // This is the constant value to lead every frame with per the LIN specification.
        // In bits, this is "10101010" or "0x55" in hex.
        frame[0] = 0x55;

        frame[1] = id;
        frame[2..(2 + data.len())].copy_from_slice(data);
        frame[2 + data.len()] = checksum;

        // Send the break signal
        match self.send_break() {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        // Write the frame to the UART
        match self.uart.write(&frame) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        // Delay to ensure the frame is sent
        match self.config.inter_frame_space {
            LinInterFrameSpace::None => (),
            LinInterFrameSpace::DelayMicroseconds(us) => self.delay.delay_ns(us as u32 * 1_000),
            LinInterFrameSpace::DelayMilliseconds(ms) => self.delay.delay_ns(ms as u32 * 1_000_000),
        }

        Ok(())
    }

    /// Read a frame from the LIN bus with the given ID into the buffer.
    /// Returns the number of bytes read into the buffer.
    pub fn read_frame(&mut self, id: u8, buffer: &mut [u8]) -> Result<usize, E> {
        // Send the break signal
        match self.send_break() {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        // Write the header to UART
        let header = [0x55, id];
        match self.uart.write(&header) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        // Delay to ensure the header has time to be received and responded to
        match self.config.read_device_response_timeout {
            LinReadDeviceResponseTimeout::None => (),
            LinReadDeviceResponseTimeout::DelayMicroseconds(us) => {
                self.delay.delay_ns(us as u32 * 1_000)
            }
            LinReadDeviceResponseTimeout::DelayMilliseconds(ms) => {
                self.delay.delay_ns(ms as u32 * 1_000_000)
            }
        }

        // Read the frame from UART
        let len;
        match self.uart.read(buffer) {
            Ok(n) => len = n,
            Err(e) => return Err(e),
        }

        // Delay to ensure the frame is read
        match self.config.inter_frame_space {
            LinInterFrameSpace::None => (),
            LinInterFrameSpace::DelayMicroseconds(us) => self.delay.delay_ns(us as u32 * 1_000),
            LinInterFrameSpace::DelayMilliseconds(ms) => self.delay.delay_ns(ms as u32 * 1_000_000),
        }

        Ok(len)
    }
}
