pub fn get_le_bit(b: &[u8; 32], index: usize) -> bool {
    b[index >> 3] & (1_u8 << (index & 7)) != 0
}

pub fn set_le_bit(b: &mut [u8; 32], index: usize) {
    b[index >> 3] |= 1_u8 << (index & 7);
}

pub fn clear_le_bit(b: &mut [u8; 32], index: usize) {
    b[index >> 3] &= !(1_u8 << (index & 7));
}

pub fn flip_le_bit(b: &mut [u8; 32], index: usize) {
    b[index >> 3] ^= 1_u8 << (index & 7);
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::encode as hex;

    #[test]
    fn test_bit_manipulation() {
        let mut u = [0_u8; 32];
        set_le_bit(&mut u, 0);
        assert_eq!(hex(&u), "0100000000000000000000000000000000000000000000000000000000000000");
        set_le_bit(&mut u, 255);
        assert_eq!(hex(&u), "0100000000000000000000000000000000000000000000000000000000000080");
        for i in 0..256 {
            set_le_bit(&mut u, i);
        }
        assert_eq!(hex(&u), "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

        clear_le_bit(&mut u, 0);
        assert_eq!(hex(&u), "feffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        clear_le_bit(&mut u, 255);
        assert_eq!(hex(&u), "feffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f");
        clear_le_bit(&mut u, 126);
        assert_eq!(hex(&u), "feffffffffffffffffffffffffffffbfffffffffffffffffffffffffffffff7f");
        for i in 0..256 {
            clear_le_bit(&mut u, i);
        }
        assert_eq!(hex(&u), "0000000000000000000000000000000000000000000000000000000000000000");

        flip_le_bit(&mut u, 0);
        assert_eq!(hex(&u), "0100000000000000000000000000000000000000000000000000000000000000");
        flip_le_bit(&mut u, 255);
        assert_eq!(hex(&u), "0100000000000000000000000000000000000000000000000000000000000080");
        flip_le_bit(&mut u, 255);
        assert_eq!(hex(&u), "0100000000000000000000000000000000000000000000000000000000000000");
        for i in 0..256 {
            flip_le_bit(&mut u, i);
        }
        assert_eq!(hex(&u), "feffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    }
}
