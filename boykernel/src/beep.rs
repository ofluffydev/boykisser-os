use x86::io::{inb, outb};

use crate::utils::sleep;

const PIT_CHANNEL2: u16 = 0x42;
const PIT_COMMAND: u16 = 0x43;
const SPEAKER_PORT: u16 = 0x61;

/// Beep at `freq` Hz for `duration_ms` milliseconds.
pub fn beep(freq: u32, duration_ms: u64) {
    let divisor = 1193180 / freq;

    unsafe {
        // Set PIT to mode 3 (square wave) on channel 2
        outb(PIT_COMMAND, 0xB6);
        outb(PIT_CHANNEL2, (divisor & 0xFF) as u8); // Low byte
        outb(PIT_CHANNEL2, ((divisor >> 8) & 0xFF) as u8); // High byte

        // Enable speaker
        let tmp = inb(SPEAKER_PORT);
        outb(SPEAKER_PORT, tmp | 3); // Enable speaker (set bits 0 and 1)
    }

    // Sleep (you'll need a way to sleep or spin)
    sleep(duration_ms);

    unsafe {
        // Turn off speaker
        let tmp = inb(SPEAKER_PORT) & 0xFC;
        outb(SPEAKER_PORT, tmp);
    }
}
