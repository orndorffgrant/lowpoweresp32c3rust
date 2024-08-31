#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*,
    systimer::SystemTimer, Delay, Rng,
};

use embedded_io::*;
use esp_wifi::wifi::{AccessPointInfo, AuthMethod, ClientConfiguration, Configuration};

use esp_backtrace as _;
use esp_println::{print, println};
use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi::{WifiError, WifiStaDevice};
use esp_wifi::wifi_interface::WifiStack;
use esp_wifi::{current_millis, initialize, EspWifiInitFor};
use smoltcp::iface::SocketStorage;
use smoltcp::wire::IpAddress;
use smoltcp::wire::Ipv4Address;

use sensor_temp_humidity_sht40::{I2CAddr, Precision, SHT40Driver, TempUnit};

const SSID: &str = env!("ENPM818L_SSID");
const PASSWORD: &str = env!("ENPM818L_PASSWORD");

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    // Set clocks at maximum frequency
    let clocks = ClockControl::max(system.clock_control).freeze();

    // Set up some resources we need
    let mut delay = Delay::new(&clocks);
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        100u32.kHz(),
        &clocks,
    );
    let mut sht40 = SHT40Driver::new(i2c, I2CAddr::SHT4x_A, delay);

    // Initialize the timers used for Wifi
    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    // Configure Wifi
    let wifi = peripherals.WIFI;
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let (iface, device, mut controller, sockets) =
        create_network_interface(&init, wifi, WifiStaDevice, &mut socket_set_entries).unwrap();

    println!("Configured to connect to {}", SSID);
    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });

    let res = controller.set_configuration(&client_config);
    println!("Wi-Fi set_configuration returned {:?}", res);

    controller.start().unwrap();
    println!("Is wifi started: {:?}", controller.is_started());

    println!("Scan WiFi until we find SSID: {}", SSID);
    let mut found_ssid = false;
    while !found_ssid {
        let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> =
            controller.scan_n();
        if let Ok((res, _count)) = res {
            println!("Found:");
            for ap in res {
                println!("  - {}", ap.ssid);
                if ap.ssid == SSID {
                    found_ssid = true;
                }
            }
        }
        if !found_ssid {
            println!("Didn't find SSID, waiting 3 seconds before trying again...");
            delay.delay_ms(3000u32)
        }
    }

    let mut connected_success = false;
    while !connected_success {
        println!("Trying to connect");
        let _ = controller.connect();
        println!("Wait to get connected");
        connected_success = loop {
            let res = controller.is_connected();
            match res {
                Ok(connected) => {
                    if connected {
                        break true;
                    }
                }
                Err(err) => {
                    println!("Error connecting: {:?}", err);
                    break false;
                }
            }
        };
        if !connected_success {
            println!("Failed to connect, waiting 3 seconds before trying again...");
            delay.delay_ms(3000u32)
        }
    }

    println!("Connected: {:?}", controller.is_connected());

    // Wait for getting an ip address
    let wifi_stack = WifiStack::new(iface, device, sockets, current_millis);
    println!("Wait to get an ip address");
    loop {
        wifi_stack.work();

        if wifi_stack.is_iface_up() {
            println!("Got IP: {}", wifi_stack.get_ip_info().unwrap().ip);
            break;
        }
    }

    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 1536];
    let mut socket = wifi_stack.get_socket(&mut rx_buffer, &mut tx_buffer);

    loop {
        println!("Getting temperature data from sensor");
        let reading = sht40.get_temp_and_rh(Precision::High, TempUnit::MilliDegreesFahrenheit).unwrap();
        println!("Got reading: {:?}", reading);
        println!("Sending temperature data to server");
        socket.work();

        socket
            .open(IpAddress::Ipv4(Ipv4Address::new(10, 42, 0, 1)), 4000)
            .unwrap();

        let mut buffer = itoa::Buffer::new();
        let printed = buffer.format(reading.temp);
        socket.write(printed.as_bytes()).unwrap();
        socket.flush().unwrap();

        let wait_end = current_millis() + 1 * 1000;
        print!("Response: \"");
        loop {
            let mut buffer = [0u8; 512];
            if let Ok(len) = socket.read(&mut buffer) {
                let to_print = unsafe { core::str::from_utf8_unchecked(&buffer[..len]) };
                print!("{}", to_print);
            } else {
                break;
            }

            if current_millis() > wait_end {
                println!("Timeout");
                break;
            }
        }
        println!("\"");

        socket.disconnect();

        let wait_end = current_millis() + 5 * 1000;
        while current_millis() < wait_end {
            socket.work();
        }
    }
}
