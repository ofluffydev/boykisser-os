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

/// Sleep for the specified number of milliseconds
pub fn sleep(milliseconds: u64) {
    const CPU_FREQUENCY_HZ: u64 = 2_200_000_000;
    let cycles_per_ms = CPU_FREQUENCY_HZ / 1_000;
    let target = read_timestamp_counter() + (milliseconds * cycles_per_ms);

    while read_timestamp_counter() < target {
        // Reduce CPU power consumption while spinning
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}
