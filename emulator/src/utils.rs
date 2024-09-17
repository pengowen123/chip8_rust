//! Utility functions

/// Returns the number in its BCD representation
/// The most significant digit is stored first in the result
pub fn bcd(num: u8) -> [u8; 3] {
    [num / 100 % 10, num / 10 % 10, num % 10]
}

#[cfg(test)]
mod tests {
    use super::bcd;

    #[test]
    fn test_bcd() {
        for num in 0..255 {
            let bcd = bcd(num);
            assert_eq!(num, bcd[0] * 100 + bcd[1] * 10 + bcd[2]);
        }
    }
}
