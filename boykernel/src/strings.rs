extern crate alloc;

use alloc::string::String;

/// Safely concatenates two strings in a no_std safe and compatible way
pub fn concat(string1: &str, string2: &str) -> String {
    let mut result = String::with_capacity(string1.len() + string2.len());
    result.push_str(string1);
    result.push_str(string2);
    result
}