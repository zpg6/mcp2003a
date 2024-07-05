//! MCP2003A LIN Transceiver Library
//!
//! ⚠️ WORK IN PROGRESS
//!
//! This library provides an `embedded-hal` abstraction for the MCP2003A LIN transceiver using UART
//! and a GPIO output pin for the break signal.
//!
//! LIN (Local Interconnect Network) is a serial network protocol used in automotive and industrial applications.
//! Most automobiles on the road today have several LIN bus networks for various systems like climate control,
//! power windows, lighting, and more.
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

#![no_std]

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal_nb::serial::{Read as UartRead, Write as UartWrite};

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

/// LIN Wakeup Signal Duration for the MCP2003A transceiver.
/// The specification requires a minimum of 250 microseconds for the wakeup signal.
#[derive(Clone, Copy, Debug)]
pub enum LinWakeupDuration {
    Minimum250Microseconds,
    Minimum250MicrosecondsPlus(u32),
    Maximum5Milliseconds,
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
    /// LIN Bus Speed / Baud Rate in bits per second.
    pub speed: LinBusSpeed,
    /// Duration of the break signal at the beginning of a frame.
    pub break_duration: LinBreakDuration,
    /// Duration of the wakeup signal at the beginning of communication.
    pub wakeup_duration: LinWakeupDuration,
    /// How long to wait after sending a read header before reading the response from the device.
    pub read_device_response_timeout: LinReadDeviceResponseTimeout,
    /// How long to wait after sending a frame before sending the next frame.
    pub inter_frame_space: LinInterFrameSpace,
}

impl Default for LinBusConfig {
    fn default() -> Self {
        LinBusConfig {
            speed: LinBusSpeed::Baud19200,
            break_duration: LinBreakDuration::Minimum13Bits,
            wakeup_duration: LinWakeupDuration::Minimum250Microseconds,
            read_device_response_timeout: LinReadDeviceResponseTimeout::DelayMilliseconds(2),
            inter_frame_space: LinInterFrameSpace::DelayMilliseconds(1),
        }
    }
}

#[derive(Debug)]
pub enum Mcp2003aError<E> {
    UartError(embedded_hal_nb::nb::Error<E>),
    UartWriteNotReady,
    LinDeviceNoResponse,
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
    fn send_break(&mut self) {
        // Calculate the duration of the break signal
        let bit_period_ns = self.config.speed.get_bit_period_ns();
        let break_duration_ns = self.config.break_duration.get_duration_ns(bit_period_ns);

        // Start the break
        self.break_pin.set_high().unwrap();

        // Break for the duration based on baud rate
        self.delay.delay_ns(break_duration_ns);

        // End the break
        self.break_pin.set_low().unwrap();

        // Break delimiter is 1 bit time
        self.delay.delay_ns(bit_period_ns);
    }

    /// Send a wakeup signal on the LIN bus, pausing execution for at least 250 microseconds.
    ///
    /// Note: there is an additional delay of the configured wakeup duration after the wakeup signal
    /// to ensure the bus devices are ready to receive frames after activation.
    pub fn send_wakeup(&mut self) {
        // Calculate the duration of the wakeup signal
        let wakeup_duration_ns = match self.config.wakeup_duration {
            LinWakeupDuration::Minimum250Microseconds => 250_000,
            LinWakeupDuration::Minimum250MicrosecondsPlus(us) => 250_000 + us,
            LinWakeupDuration::Maximum5Milliseconds => 5_000_000,
        };

        // Ensure the wakeup duration is less than 5 milliseconds
        assert!(
            wakeup_duration_ns <= 5_000_000,
            "Wakeup duration must be less than 5 milliseconds"
        );

        // Start the wakeup signal
        self.break_pin.set_high().unwrap();

        // Wakeup for the duration
        self.delay.delay_ns(wakeup_duration_ns);

        // End the wakeup signal
        self.break_pin.set_low().unwrap();

        // Delay after wakeup signal
        self.delay.delay_ns(wakeup_duration_ns);
    }

    /// Send a frame on the LIN bus with the given ID, data, and checksum.
    /// The data length must be between 0 and 8 bytes.
    ///
    /// Note: Inter-frame space is applied after sending the frame.
    pub fn send_frame(
        &mut self,
        id: u8,
        data: &[u8],
        checksum: u8,
    ) -> Result<[u8; 11], Mcp2003aError<E>> {
        // Calculate the length of the data
        assert!(
            data.len() <= 8 && data.len() > 0,
            "Data length must be between 1 and 8 bytes"
        );
        let data_len = data.len();

        // Calculate the frame
        let mut frame = [0; 11];

        // This is the constant value to lead every frame with per the LIN specification.
        // In bits, this is "10101010" or "0x55" in hex.
        frame[0] = 0x55;

        frame[1] = id;
        frame[2..2 + data_len].copy_from_slice(data);
        frame[2 + data_len] = checksum;

        // Send the break signal
        self.send_break();

        // Write the frame to the UART
        for byte in frame.iter() {
            match self.uart.write(*byte) {
                Ok(_) => (),
                Err(e) => return Err(Mcp2003aError::UartError(e)),
            }
        }

        // Inter-frame space delay
        match self.config.inter_frame_space {
            LinInterFrameSpace::None => (),
            LinInterFrameSpace::DelayMicroseconds(us) => self.delay.delay_ns(us as u32 * 1_000),
            LinInterFrameSpace::DelayMilliseconds(ms) => self.delay.delay_ns(ms as u32 * 1_000_000),
        }

        Ok(frame)
    }

    /// Read a frame from the LIN bus with the given ID into the buffer.
    /// Returns the number of bytes read into the buffer.
    ///
    /// Note: Inter-frame space is applied after reading the frame.
    pub fn read_frame(&mut self, id: u8, buffer: &mut [u8]) -> Result<usize, Mcp2003aError<E>> {
        // Send the break signal to notify the device of the start of a frame
        self.send_break();

        // Write the header to UART
        let header = [0x55, id];
        for byte in header.iter() {
            match self.uart.write(*byte) {
                Ok(_) => (),
                Err(e) => return Err(Mcp2003aError::UartError(e)),
            }
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

        // Read the response from the device
        let mut len = 0;
        while len < buffer.len() {
            match self.uart.read() {
                Ok(byte) => {
                    buffer[len] = byte;
                    len += 1;
                }
                Err(_) => break,
            }
        }

        // Delay to ensure the frame is read
        match self.config.inter_frame_space {
            LinInterFrameSpace::None => (),
            LinInterFrameSpace::DelayMicroseconds(us) => self.delay.delay_ns(us as u32 * 1_000),
            LinInterFrameSpace::DelayMilliseconds(ms) => self.delay.delay_ns(ms as u32 * 1_000_000),
        }

        if len == 0 {
            return Err(Mcp2003aError::LinDeviceNoResponse);
        }

        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_break_duration() {
        let mut config = LinBusConfig {
            speed: LinBusSpeed::Baud19200,
            break_duration: LinBreakDuration::Minimum13Bits,
            wakeup_duration: LinWakeupDuration::Minimum250Microseconds,
            read_device_response_timeout: LinReadDeviceResponseTimeout::DelayMilliseconds(2),
            inter_frame_space: LinInterFrameSpace::DelayMilliseconds(1),
        };

        assert_eq!(config.break_duration.get_duration_ns(52_083), 677_079);

        config.break_duration = LinBreakDuration::Minimum13BitsPlus(1);
        assert_eq!(config.break_duration.get_duration_ns(52_083), 729_162);

        config.break_duration = LinBreakDuration::Minimum13BitsPlus(2);
        assert_eq!(config.break_duration.get_duration_ns(52_083), 781_245);
    }

    #[test]
    fn test_speed() {
        let speed = LinBusSpeed::Baud19200;
        assert_eq!(speed.get_baud_rate(), 19200);
        assert_eq!(speed.get_bit_period_ns(), 52_083);
    }
}
