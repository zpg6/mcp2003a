use esp_idf_svc::hal::{
    delay::{Ets, FreeRtos},
    gpio::{Gpio0, Gpio1, PinDriver},
    peripherals::Peripherals,
    uart::{
        config::{Config as UartConfig, DataBits, StopBits},
        UartDriver,
    },
    units::Hertz,
};
use mcp2003a::{
    config::{
        LinBreakDuration, LinBusConfig, LinBusSpeed, LinInterFrameSpace, LinReadDeviceResponseTimeout,
        LinWakeupDuration,
    },
    Mcp2003a, Mcp2003aError,
};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Take the peripherals from the ESP-IDF framework
    let peripherals = Peripherals::take().unwrap();

    // Configure UART2 for sending and receiving LIN Bus frames between
    // this and the MCP2003A LIN Transceiver.
    let uart2 = peripherals.uart2;
    let uart2_rx = peripherals.pins.gpio16;
    let uart2_tx = peripherals.pins.gpio17;
    let uart2_config = UartConfig::default()
        .baudrate(Hertz(19200)) // Configure the baudrate for the LIN Bus here as well
        .data_bits(DataBits::DataBits8)
        .parity_none()
        .stop_bits(StopBits::STOP1);
    let uart2_driver = UartDriver::new(
        uart2,
        uart2_tx,
        uart2_rx,
        Option::<Gpio0>::None,
        Option::<Gpio1>::None,
        &uart2_config,
    )
    .unwrap();

    // Configure the GPIO pin for the LIN Bus break signal
    let break_pin = peripherals.pins.gpio15;
    let break_pin_driver = PinDriver::output(break_pin).unwrap();

    // Use Ets to delay for small periods of time
    let delay = Ets;

    // Configure the LIN Bus with the following parameters:
    let lin_bus_config = LinBusConfig {
        speed: LinBusSpeed::Baud19200,
        break_duration: LinBreakDuration::Minimum13Bits, // Test for your application
        wakeup_duration: LinWakeupDuration::Minimum250Microseconds, // Test for your application
        read_device_response_timeout: LinReadDeviceResponseTimeout::DelayMilliseconds(2), // Test for your application
        inter_frame_space: LinInterFrameSpace::DelayMilliseconds(1), // Test for your application
    };

    // Initialize the MCP2003A LIN Transceiver
    let mut mcp2003a = Mcp2003a::new(uart2_driver, break_pin_driver, delay);
    mcp2003a.init(lin_bus_config);
    log::info!("MCP2003A LIN Transceiver initialized");

    // Wakeup the LIN Bus
    mcp2003a.send_wakeup();

    loop {
        // Send a frame on the LIN bus to a device with Command frame of 0x00:
        // - LIN Id: 0x00 --> PID: 0x80
        // - Data: [0x00, 0xF0, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x08]
        // - Checksum: 0x7C
        match mcp2003a.send_frame(0x80, &[0x00, 0xF0, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x08], 0x7C) {
            Ok(frame) => {
                // Frame sent
                log::info!("Sent data to LIN Id 0x00: {:?}", frame);
            }
            Err(e) => {
                // Error sending the frame
                log::error!("Error sending frame: {:?}", e);
            }
        }

        // Over-sending not needed, so delay between frames
        FreeRtos::delay_ms(500);

        // Read the feedback / diagnostic frame 0x01 from the LIN bus:
        // - LIN Id: 0x01 --> PID: 0xC1
        // - Data: Buffer of 8 bytes will explicitly try to read 8 bytes then a checksum
        let mut data = [0u8; 8];
        match mcp2003a.read_frame(0xC1, &mut data) {
            Ok(checksum) => {
                // Data is stored in the buffer
                log::info!("Received data from LIN Id 0x01: {:?} with checksum: 0x{:02X}", data, checksum);
            }
            Err(e) => {
                // Error reading the frame
                match e {
                    Mcp2003aError::LinDeviceTimeoutNoResponse => {
                        log::warn!("No response from LIN Id 0x01... this device may be offline.");
                    }
                    _ => {
                        log::error!("Error reading frame: {:?}", e);
                    }
                }
            }
        }

        // Over-sending not needed, so delay between frames
        FreeRtos::delay_ms(500);
    }
}
