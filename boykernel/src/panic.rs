use crate::{gop_render::CURSOR_STATE, serial::serial_write_str};
use crate::gop_render::SimplifiedRenderer;
use crate::append_number_to_string;
use crate::get_and_lock_renderer;
use heapless::String;

#[cfg(not(test))]
#[panic_handler]
fn panic(panic: &core::panic::PanicInfo) -> ! {
    serial_write_str("Panic occurred: ");
    print_panic_info_serial(&panic);

    unsafe {
        CURSOR_STATE.force_unlock(); // rare case the item is locked
    }
    CURSOR_STATE.lock().x = 40;
    CURSOR_STATE.lock().y = 40;

    let mut message: String<128> = String::new();
    message
        .push_str(panic.message().as_str().unwrap_or("unknown error"))
        .unwrap();

    if let Some(location) = panic.location() {
        message.push_str(" at ").unwrap();

        // Ensure the file name is valid UTF-8
        if let Ok(file_name) = core::str::from_utf8(location.file().as_bytes()) {
            message.push_str(file_name).unwrap();
        } else {
            message.push_str("<invalid file>").unwrap();
        }

        message.push_str(": Line ").unwrap(); // Add "Line" label

        let mut line_str: String<16> = String::new();
        append_number_to_string(&mut line_str, location.line() as usize);
        message.push_str(&line_str).unwrap();
    }

    get_and_lock_renderer().clear_screen();
    get_and_lock_renderer().println(&message);

    loop {
        unsafe { core::arch::asm!("hlt") }
    }
}

pub fn print_panic_info_serial(info: &core::panic::PanicInfo) {
    serial_write_str("=== PANIC ===\n");

    #[cfg(not(debug_assertions))]
    serial_write_str("[WARNING] This is a release build, panic information may be limited.\n");

    if let Some(location) = info.location() {
        serial_write_str("Location: ");
        serial_write_str(location.file());
        serial_write_str(":");
        serial_write_str(location.line().to_string().as_str());
        serial_write_str(":");
        serial_write_str(location.column().to_string().as_str());
        serial_write_str("\n");
    } else {
        serial_write_str("Location: <unknown>\n");
    }

    if let Some(message) = info.message().as_str() {
        serial_write_str("Message: ");
        serial_write_str(message);
        serial_write_str("\n");
    } else {
        serial_write_str("Message: <none>\n");
    }

    serial_write_str("=============\n");
    serial_write_str("\n\n");
}