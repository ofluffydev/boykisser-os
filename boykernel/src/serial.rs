use x86_64::instructions::port::Port;

const SERIAL_PORT: u16 = 0x3F8; // COM1

pub fn serial_write_byte(byte: u8) {
    unsafe {
        let mut line_status = Port::<u8>::new(SERIAL_PORT + 5);
        while (line_status.read() & 0x20) == 0 {} // Wait until empty

        let mut data = Port::new(SERIAL_PORT);
        data.write(byte);
    }
}

pub fn serial_write_str(s: &str) {
    for byte in s.bytes() {
        serial_write_byte(byte);
    }
}

pub fn info(text: &str) {
    serial_write_str("[INFO] ");
    serial_write_str(text);
    serial_write_str("\n");
}

#[allow(dead_code)]
pub fn error(text: &str) {
    serial_write_str("[ERROR] ");
    serial_write_str(text);
    serial_write_str("\n");
}
