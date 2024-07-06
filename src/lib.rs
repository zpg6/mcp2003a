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
use embedded_hal_nb::{
    nb::block,
    serial::{Read as UartRead, Write as UartWrite},
};

pub mod config;
use config::*;

#[derive(Debug)]
pub enum Mcp2003aError<E> {
    UartError(embedded_hal_nb::nb::Error<E>),
    UartWriteNotReady,
    LinDeviceTimeoutNoResponse,
    LinReadInvalidHeader([u8; 100]),
    LinReadInvalidChecksum,
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
        let wakeup_duration_ns = self.config.wakeup_duration.get_duration_ns();

        // Ensure the wakeup duration is less than 5 milliseconds
        assert!(wakeup_duration_ns <= 5_000_000, "Wakeup duration must be less than 5 milliseconds");

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
    pub fn send_frame(&mut self, id: u8, data: &[u8], checksum: u8) -> Result<[u8; 11], Mcp2003aError<E>> {
        // Calculate the length of the data
        assert!(data.len() <= 8 && data.len() > 0, "Data length must be between 1 and 8 bytes");
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

        // Ensures that none of the previously written words are still buffered
        match block!(self.uart.flush()) {
            Ok(_) => (),
            Err(_) => return Err(Mcp2003aError::UartWriteNotReady),
        }

        // Inter-frame space delay
        self.delay.delay_ns(self.config.inter_frame_space.get_duration_ns());

        Ok(frame)
    }

    /// Read a frame from the LIN bus with the given ID into the buffer.
    /// Returns the number of bytes read into the buffer.
    ///
    /// Note: Inter-frame space is applied after reading the frame.
    pub fn read_frame(&mut self, id: u8, buffer: &mut [u8]) -> Result<usize, Mcp2003aError<E>> {
        // Inter-frame space delay
        self.delay.delay_ns(self.config.inter_frame_space.get_duration_ns());

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

        // Delay to ensure the header has time to be received and responded to by the device
        self.delay.delay_ns(self.config.read_device_response_timeout.get_duration_ns());

        // Read the response from the device
        // NOTE: The mcp2003a will replay the header back to you when you read.
        let mut len = 0;

        while len < buffer.len() {
            match self.uart.read() {
                Ok(byte) => {
                    // While there are some bytes in the uart buffer,
                    // keep skipping until we find the header [0x55, id]

                    if len == 0 && byte != 0x55 {
                        continue;
                    }
                    if len == 1 && byte != id {
                        // Start over recording if response doesn't start with [0x55, id]
                        len = 0;
                    }

                    buffer[len] = byte;
                    len += 1;
                }
                Err(embedded_hal_nb::nb::Error::WouldBlock) => {
                    // If we get a WouldBlock error, we've read all the bytes in the buffer
                    break;
                }
                Err(e) => return Err(Mcp2003aError::UartError(e)),
            }
        }

        // Inter-frame space delay
        self.delay.delay_ns(self.config.inter_frame_space.get_duration_ns());

        // Assume empty response is a timeout
        if len <= 2 {
            return Err(Mcp2003aError::LinDeviceTimeoutNoResponse);
        }

        Ok(len)
    }
}
