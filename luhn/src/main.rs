// TODO: remove this when you're done with your implementation.
#![allow(unused_variables, dead_code)]

pub fn luhn(cc_number: &str) -> bool {
    println!("{x:?}", x=cc_number);
    // remove whitespace
    let cc_chars: String = cc_number.chars().filter(|c| !c.is_whitespace()).collect();
    println!("{x:?}", x=cc_chars);
    // check if long enough
    if cc_chars.len() < 2 { return false; };
    // check if all digits
    if !cc_chars.chars().all(char::is_numeric) { return false; };
    
    // convert into vector of int
    const RADIX: u32 = 10;
    let mut cc_ints: Vec<u32> = cc_chars.chars().rev().map(|c| c.to_digit(RADIX).unwrap()).collect();
    println!("{x:?}", x=cc_ints);
    for digit in cc_ints.iter_mut().skip(1).step_by(2) {
        let checksum = (*digit * 2).to_string().chars().map(|c| c.to_digit(RADIX).unwrap()).sum::<u32>();
        *digit = checksum;
    }
    println!("{x:?}", x=cc_ints);
    // sum all digits
    let final_checksum: u32 = cc_ints.iter().sum();
    println!("{x:?}", x=final_checksum);
    // cc is valid if checksum ends with 0
    final_checksum % 10 == 0
}

#[test]
fn test_non_digit_cc_number() {
    assert!(!luhn("foo"));
}

#[test]
fn test_empty_cc_number() {
    assert!(!luhn(""));
    assert!(!luhn(" "));
    assert!(!luhn("  "));
    assert!(!luhn("    "));
}

#[test]
fn test_single_digit_cc_number() {
    assert!(!luhn("0"));
}

#[test]
fn test_two_digit_cc_number() {
    assert!(luhn(" 0 0 "));
}

#[test]
fn test_valid_cc_number() {
    assert!(luhn("4263 9826 4026 9299"));
    assert!(luhn("4539 3195 0343 6467"));
    assert!(luhn("7992 7398 713"));
}

#[test]
fn test_invalid_cc_number() {
    assert!(!luhn("4223 9826 4026 9299"));
    assert!(!luhn("4539 3195 0343 6476"));
    assert!(!luhn("8273 1232 7352 0569"));
}

#[allow(dead_code)]
fn main() {}