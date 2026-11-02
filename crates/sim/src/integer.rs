//! Deterministic integer-only math helpers used by map generation and combat.

/// Returns floor(sqrt(n)) for a u128 without floating-point operations.
pub fn integer_sqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let bits = u128::BITS - n.leading_zeros();
    let mut estimate = 1_u128 << bits.div_ceil(2);
    loop {
        let next = (estimate + n / estimate) / 2;
        if next >= estimate {
            return estimate;
        }
        estimate = next;
    }
}

#[cfg(test)]
mod tests {
    use super::integer_sqrt;

    #[test]
    fn matches_reference_values() {
        let cases = [
            (0, 0),
            (1, 1),
            (2, 1),
            (3, 1),
            (4, 2),
            (8, 2),
            (9, 3),
            (15, 3),
            (16, 4),
            (17, 4),
            (1_000_000, 1_000),
            (u64::MAX as u128, u32::MAX as u128),
        ];
        for (input, expected) in cases {
            assert_eq!(integer_sqrt(input), expected, "sqrt({input})");
        }
    }

    #[test]
    fn result_is_the_floor_square_root_at_boundaries() {
        for root in [2_u128, 3, 10, 1_000, u32::MAX as u128, u64::MAX as u128] {
            let square = root * root;
            assert_eq!(integer_sqrt(square), root);
            if square < u128::MAX {
                assert_eq!(integer_sqrt(square + 1), root);
            }
        }
        assert_eq!(integer_sqrt(u128::MAX), u64::MAX as u128);
    }
}
