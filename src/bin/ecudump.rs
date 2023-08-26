use std::{error::Error, fs::File, io::Write};

use ssm::SsmSerial;

// TODO: We got 7ED2FA (~Òú) with ignition off, what does that mean?

// TODO: FFFF97 when ignition is on. Is that ROM ID or data at 0xffff?


//ssm_reset

fn main() -> Result<(), Box<dyn Error>> {
    match serial2::SerialPort::available_ports() {
        Err(e) => {
            eprintln!("Failed to enumerate serial ports: {}", e);
            Err(e)
        },
        Ok(ports) => {
            eprintln!("Found {} ports", ports.len());
            for port in ports {
                println!("{}", port.display());
            }
            Ok(())
        }
    }?;
    // TODO: Allow port selection when none specified

    let mut ssms = SsmSerial::new("/dev/ttyUSB0")?;

    let mut buffer = [0; 0x10000];

    'outer: for i in 0..=0xffff {
        println!("addr {}", i);
        for _retry in 0..3 {
            match ssms.read_mem_ecu(i) {
                Ok(v) => {
                    buffer[i as usize] = v;
                    continue 'outer;
                },
                Err(e) => {
                    eprintln!("Got error {} at {}", e, i);
                    // Stop ECU responses completely, then go for the next try
                    ssms.stop()?;
                    continue;
                }
            }
        }
        // Out of retries
        panic!("Out of retries at addr {}", i);
    }

    let mut file = File::create("ecu_fulldump.bin")?;
    file.write_all(&buffer)?;

    println!("Done");

    Ok(())
}
