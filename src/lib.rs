use std::{path::Path, time::Duration};

use serial2::SerialPort;

pub struct SsmSerial {
    port: SerialPort,
    last_addr: u32,
}

impl SsmSerial {
    pub fn new (port_name: impl AsRef<Path>) -> std::io::Result<Self> {
        let mut port = SerialPort::open(port_name, |mut s: serial2::Settings| -> std::io::Result<serial2::Settings> {
            s.set_raw();
            s.set_baud_rate(1953)?;
            s.set_char_size(serial2::CharSize::Bits8);
            s.set_stop_bits(serial2::StopBits::One);
            s.set_parity(serial2::Parity::Even);
            s.set_flow_control(serial2::FlowControl::None);
            Ok(s)
        })?;

        port.set_read_timeout(Duration::from_secs(1))?;

        Ok(Self {
            port: port,
            last_addr: 0xffffffff,
        })
    }

    pub fn read_mem_ecu (&mut self, addr: u16) -> std::io::Result<u8> {
        let cmd_buf: [u8; 4] = [
            0x78,
            (addr >> 8) as u8,
            (addr & 0xFF) as u8,
            0x00,
        ];

        self.port.write(&cmd_buf)?;

        let mut buffer = [0; 3];
        'retries: for _retry in 0..1000 {
            for i in 0..3 {
                let mut tbuf = [0; 1];
                match self.port.read(&mut tbuf) {
                    Ok(n) => {
                        if n != 1 {
                            unreachable!();
                        }

                        buffer[i] = tbuf[0];

                        if i == 2 {
                            let a = (buffer[0] as u16) << 8 | buffer[1] as u16;
                            if a != addr {
                                if a as u32 == self.last_addr {
                                    // Stale response, read again
                                    continue 'retries;
                                } else {
                                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Response address does not match requested address"));
                                }
                            }

                            self.last_addr = addr as u32;
                            return Ok(buffer[2]);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }

                }
            }
        }

        Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "No valid response to read command"))
    }

    pub fn stop (&self) -> std::io::Result<()> {
        let cmd_buf: [u8; 4] = [0x12, 0x00, 0x00, 0x00];

        for _retry in 0..100 {
            // TODO: Maybe reduce the amount of stop commands we send?
            self.port.write(&cmd_buf)?;

            let mut buf = [0; 3];

            match self.port.read(&mut buf) {
                Ok(_n) => continue,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    break;
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}
