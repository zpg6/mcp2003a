/// LIN Break Duration for the MCP2003A transceiver.
/// The specification requires a minimum of 13 bits for the break signal, but the actual underlying
/// implementation of the LIN devices may require more bits for stability (maybe 13 bits + 1 or 2 bits).
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinWakeupDuration {
    Minimum250Microseconds,
    Minimum250MicrosecondsPlus(u32),
    Maximum5Milliseconds,
}

/// How long to wait after sending a read header before reading the response, allowing the slave device to respond.
/// Typically this is a 1-10 ms delay but can vary by system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinReadDeviceResponseTimeout {
    None,
    DelayMicroseconds(u32),
    DelayMilliseconds(u32),
}

/// How long to wait after sending a frame before sending the next frame.
/// This applies to both sending and receiving frames. Typically, this is 1-2 ms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LinInterFrameSpace {
    None,
    DelayMicroseconds(u32),
    DelayMilliseconds(u32),
}

/// LIN Bus Speeds available for the MCP2003A transceiver in bits per second.
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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

    #[test]
    fn test_default_config() {
        let config = LinBusConfig::default();
        assert_eq!(config.speed, LinBusSpeed::Baud19200);
        assert_eq!(config.break_duration, LinBreakDuration::Minimum13Bits);
        assert_eq!(config.wakeup_duration, LinWakeupDuration::Minimum250Microseconds);
        assert_eq!(config.read_device_response_timeout, LinReadDeviceResponseTimeout::DelayMilliseconds(2));
        assert_eq!(config.inter_frame_space, LinInterFrameSpace::DelayMilliseconds(1));
    }
}
