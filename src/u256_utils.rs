use uint::U256;

pub fn set_bit(u256: &mut U256, index: usize) {
    u256.0[index / 64] |= 1u64 << (index % 64);
}

pub fn clear_bit(u256: &mut U256, index: usize) {
    u256.0[index / 64] &= !(1u64 << (index % 64));
}

pub fn flip_bit(u256: &mut U256, index: usize) {
    u256.0[index / 64] ^= 1u64 << (index % 64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_manipulation() {
        let mut u = U256::zero();
        set_bit(&mut u, 0);
        assert_eq!(hex(&u), "0000000000000000000000000000000000000000000000000000000000000001");
        set_bit(&mut u, 255);
        assert_eq!(hex(&u), "8000000000000000000000000000000000000000000000000000000000000001");
        for i in 0..256 {
            set_bit(&mut u, i);
        }
        assert_eq!(hex(&u), "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

        clear_bit(&mut u, 0);
        assert_eq!(hex(&u), "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
        clear_bit(&mut u, 255);
        assert_eq!(hex(&u), "7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
        clear_bit(&mut u, 126);
        assert_eq!(hex(&u), "7fffffffffffffffffffffffffffffffbffffffffffffffffffffffffffffffe");
        for i in 0..256 {
            clear_bit(&mut u, i);
        }
        assert_eq!(hex(&u), "0000000000000000000000000000000000000000000000000000000000000000");

        flip_bit(&mut u, 0);
        assert_eq!(hex(&u), "0000000000000000000000000000000000000000000000000000000000000001");
        flip_bit(&mut u, 255);
        assert_eq!(hex(&u), "8000000000000000000000000000000000000000000000000000000000000001");
        flip_bit(&mut u, 255);
        assert_eq!(hex(&u), "0000000000000000000000000000000000000000000000000000000000000001");
        for i in 0..256 {
            flip_bit(&mut u, i);
        }
        assert_eq!(hex(&u), "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
    }

    fn hex(u: &U256) -> String {
        // TODO: submit a PR to support format `{:064x}` for U256.
        let s = format!("{:x}", u);
        println!("s = {}", s);
        let padding_len = (64 as usize).saturating_sub(s.len());
        String::from_utf8(vec![b'0'; padding_len]).unwrap() + &s
    }
}
