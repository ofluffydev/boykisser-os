use core::arch::asm;

/// Reads the CPU's timestamp counter
pub fn read_timestamp_counter() -> u64 {
    let mut low: u32;
    let mut high: u32;
    unsafe {
        asm!(
            "rdtsc",
            out("eax") low,
            out("edx") high
        );
    }
    ((high as u64) << 32) | (low as u64)
}

/// Busy-wait loop to sleep for the specified number of milliseconds
pub fn sleep(milliseconds: u64) {
    // Assuming a CPU frequency of 3 GHz (adjust as needed for your system)
    const CPU_FREQUENCY_HZ: u64 = 2_200_000_000;
    let cycles_per_ms = CPU_FREQUENCY_HZ / 1_000;

    let start = read_timestamp_counter();
    let target = start + (milliseconds * cycles_per_ms);

    while read_timestamp_counter() < target {
        // Busy-wait
    }
}

pub fn append_number_to_string<const N: usize>(s: &mut heapless::String<N>, num: usize) {
    let mut buffer = [0u8; 20]; // Enough to hold any usize
    let mut i = 0;
    let mut n = num;

    // Convert the number to a string in reverse order
    loop {
        buffer[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
        if n == 0 {
            break;
        }
    }

    // Append the digits in the correct order
    for j in (0..i).rev() {
        let _ = s.push(buffer[j] as char);
    }
}