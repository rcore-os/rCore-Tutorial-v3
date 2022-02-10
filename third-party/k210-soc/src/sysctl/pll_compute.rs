/** PLL configuration */
#[derive(Debug, PartialEq, Eq)]
pub struct Params {
    pub clkr: u8,
    pub clkf: u8,
    pub clkod: u8,
    pub bwadj: u8,
}

/*
 * The PLL included with the Kendryte K210 appears to be a True Circuits, Inc.
 * General-Purpose PLL. The logical layout of the PLL with internal feedback is
 * approximately the following:
 *
 *  +---------------+
 *  |reference clock|
 *  +---------------+
 *          |
 *          v
 *        +--+
 *        |/r|
 *        +--+
 *          |
 *          v
 *   +-------------+
 *   |divided clock|
 *   +-------------+
 *          |
 *          v
 *  +--------------+
 *  |phase detector|<---+
 *  +--------------+    |
 *          |           |
 *          v   +--------------+
 *        +---+ |feedback clock|
 *        |VCO| +--------------+
 *        +---+         ^
 *          |    +--+   |
 *          +--->|/f|---+
 *          |    +--+
 *          v
 *        +---+
 *        |/od|
 *        +---+
 *          |
 *          v
 *       +------+
 *       |output|
 *       +------+
 *
 * The k210 PLLs have three factors: r, f, and od. Because of the feedback mode,
 * the effect of the division by f is to multiply the input frequency. The
 * equation for the output rate is
 *   rate = (rate_in * f) / (r * od).
 * Moving knowns to one side of the equation, we get
 *   rate / rate_in = f / (r * od)
 * Rearranging slightly,
 *   abs_error = abs((rate / rate_in) - (f / (r * od))).
 * To get relative, error, we divide by the expected ratio
 *   error = abs((rate / rate_in) - (f / (r * od))) / (rate / rate_in).
 * Simplifying,
 *   error = abs(1 - f / (r * od)) / (rate / rate_in)
 *   error = abs(1 - (f * rate_in) / (r * od * rate))
 * Using the constants ratio = rate / rate_in and inv_ratio = rate_in / rate,
 *   error = abs((f * inv_ratio) / (r * od) - 1)
 * This is the error used in evaluating parameters.
 *
 * r and od are four bits each, while f is six bits. Because r and od are
 * multiplied together, instead of the full 256 values possible if both bits
 * were used fully, there are only 97 distinct products. Combined with f, there
 * are 6208 theoretical settings for the PLL. However, most of these settings
 * can be ruled out immediately because they do not have the correct ratio.
 *
 * In addition to the constraint of approximating the desired ratio, parameters
 * must also keep internal pll frequencies within acceptable ranges. The divided
 * clock's minimum and maximum frequencies have a ratio of around 128. This
 * leaves fairly substantial room to work with, especially since the only
 * affected parameter is r. The VCO's minimum and maximum frequency have a ratio
 * of 5, which is considerably more restrictive.
 *
 * The r and od factors are stored in a table. This is to make it easy to find
 * the next-largest product. Some products have multiple factorizations, but
 * only when one factor has at least a 2.5x ratio to the factors of the other
 * factorization. This is because any smaller ratio would not make a difference
 * when ensuring the VCO's frequency is within spec.
 *
 * Throughout the calculation function, fixed point arithmetic is used. Because
 * the range of rate and rate_in may be up to 1.75 GHz, or around 2^30, 64-bit
 * 32.32 fixed-point numbers are used to represent ratios. In general, to
 * implement division, the numerator is first multiplied by 2^32. This gives a
 * result where the whole number part is in the upper 32 bits, and the fraction
 * is in the lower 32 bits.
 *
 * In general, rounding is done to the closest integer. This helps find the best
 * approximation for the ratio. Rounding in one direction (e.g down) could cause
 * the function to miss a better ratio with one of the parameters increased by
 * one.
 */

const VCO_MIN: u64 = 340_000_000;
const VCO_MAX: u64 = 1_750_000_000;
const DIV_MIN: u32 = 13_300_000;
const DIV_MAX: u32 = 1_750_000_000;
const R_MIN: u32 = 1;
const R_MAX: u32 = 16;
const F_MIN: u32 = 1;
const F_MAX: u32 = 64;
const OD_MIN: u32 = 1;
const OD_MAX: u32 = 16;
const OUT_MIN: u32 = (VCO_MIN as u32) / OD_MAX; /* 21_250_000 */
const OUT_MAX: u32 = (VCO_MAX as u32) / OD_MIN; /* 1_750_000_000 */
const IN_MIN: u32 = DIV_MIN * R_MIN; /* 13_300_000 */
const IN_MAX: u32 = OUT_MAX; /* 1_750_000_000 */

/*
 * The factors table was generated with the following python code:
 *
 * def p(x, y):
 *    return (1.0*x/y > 2.5) or (1.0*y/x > 2.5)
 *
 * factors = {}
 * for i in range(1, 17):
 *    for j in range(1, 17):
 *       fs = factors.get(i*j) or []
 *       if fs == [] or all([
 *             (p(i, x) and p(i, y)) or (p(j, x) and p(j, y))
 *             for (x, y) in fs]):
 *          fs.append((i, j))
 *          factors[i*j] = fs
 *
 * for k, l in sorted(factors.items()):
 *    for v in l:
 *       print("pack(%s, %s)," % v)
 */
struct Factor {
    packed: u8,
}

/* Apologies, but there are no native bitfields (yet) afaik */
const fn pack(r: u32, od: u32) -> Factor {
    Factor { packed: (((((r as u8) - 1) & 0xF) << 4) | (((od as u8) - 1) & 0xF)) }
}

const fn unpack_r(factor: &&Factor) -> u32 {
    (((factor.packed as u32) >> 4) & 0xF) + 1
}

const fn unpack_od(factor: &&Factor) -> u32 {
    ((factor.packed as u32) & 0xF) + 1
}

static FACTORS: &'static [Factor] = &[
    pack(1, 1),
    pack(1, 2),
    pack(1, 3),
    pack(1, 4),
    pack(1, 5),
    pack(1, 6),
    pack(1, 7),
    pack(1, 8),
    pack(1, 9),
    pack(3, 3),
    pack(1, 10),
    pack(1, 11),
    pack(1, 12),
    pack(3, 4),
    pack(1, 13),
    pack(1, 14),
    pack(1, 15),
    pack(3, 5),
    pack(1, 16),
    pack(4, 4),
    pack(2, 9),
    pack(2, 10),
    pack(3, 7),
    pack(2, 11),
    pack(2, 12),
    pack(5, 5),
    pack(2, 13),
    pack(3, 9),
    pack(2, 14),
    pack(2, 15),
    pack(2, 16),
    pack(3, 11),
    pack(5, 7),
    pack(3, 12),
    pack(3, 13),
    pack(4, 10),
    pack(3, 14),
    pack(4, 11),
    pack(3, 15),
    pack(3, 16),
    pack(7, 7),
    pack(5, 10),
    pack(4, 13),
    pack(6, 9),
    pack(5, 11),
    pack(4, 14),
    pack(4, 15),
    pack(7, 9),
    pack(4, 16),
    pack(5, 13),
    pack(6, 11),
    pack(5, 14),
    pack(6, 12),
    pack(5, 15),
    pack(7, 11),
    pack(6, 13),
    pack(5, 16),
    pack(9, 9),
    pack(6, 14),
    pack(8, 11),
    pack(6, 15),
    pack(7, 13),
    pack(6, 16),
    pack(7, 14),
    pack(9, 11),
    pack(10, 10),
    pack(8, 13),
    pack(7, 15),
    pack(9, 12),
    pack(10, 11),
    pack(7, 16),
    pack(9, 13),
    pack(8, 15),
    pack(11, 11),
    pack(9, 14),
    pack(8, 16),
    pack(10, 13),
    pack(11, 12),
    pack(9, 15),
    pack(10, 14),
    pack(11, 13),
    pack(9, 16),
    pack(10, 15),
    pack(11, 14),
    pack(12, 13),
    pack(10, 16),
    pack(11, 15),
    pack(12, 14),
    pack(13, 13),
    pack(11, 16),
    pack(12, 15),
    pack(13, 14),
    pack(12, 16),
    pack(13, 15),
    pack(14, 14),
    pack(13, 16),
    pack(14, 15),
    pack(14, 16),
    pack(15, 15),
    pack(15, 16),
    pack(16, 16),
];

/* Divide and round to the closest integer */
fn div_round_closest(n: u64, d: u32) -> u64 {
    let _d: u64 = d as u64;

    (n + (_d / 2)) / _d
}

/* Integer with that bit set */
fn bit(bit: u8) -> u64 {
    1 << bit
}

/* | a - b | */
fn abs_diff(a: u32, b: u32) -> u32 {
    if a > b {
        a - b
    } else {
        b - a
    }
}

pub fn compute_params(freq_in: u32, freq_out: u32) -> Option<Params> {
    let mut best: Option<Params> = None;
    let mut factors = FACTORS.iter().peekable();
    let (mut error, mut best_error): (i64, i64);
    let (ratio, inv_ratio): (u64, u64); /* fixed point 32.32 ratio of the freqs */
    let max_r: u32;
    let (mut r, mut f, mut od): (u32, u32, u32);

    /*
     * Can't go over 1.75 GHz or under 21.25 MHz due to limitations on the
     * VCO frequency. These are not the same limits as below because od can
     * reduce the output frequency by 16.
     */
    if freq_out > OUT_MAX || freq_out < OUT_MIN {
        return None;
    }

    /* Similar restrictions on the input ratio */
    if freq_in > IN_MAX || freq_in < IN_MIN {
        return None;
    }

    ratio = div_round_closest((freq_out as u64) << 32, freq_in);
    inv_ratio = div_round_closest((freq_in as u64) << 32, freq_out);
    /* Can't increase by more than 64 or reduce by more than 256 */
    if freq_out > freq_in && ratio > (64 << 32) {
        return None;
    } else if freq_out <= freq_in && inv_ratio > (256 << 32) {
        return None;
    }

    /*
     * The divided clock (freq_in / r) must stay between 1.75 GHz and 13.3
     * MHz. There is no minimum, since the only way to get a higher input
     * clock than 26 MHz is to use a clock generated by a PLL. Because PLLs
     * cannot output frequencies greater than 1.75 GHz, the minimum would
     * never be greater than one.
     */
    max_r = freq_in / DIV_MIN;

    /* Variables get immediately incremented, so start at -1th iteration */
    f = 0;
    r = 0;
    od = 0;
    best_error = i64::max_value();
    error = best_error;
    /* Always try at least one ratio */
    'outer: loop {
        /*
         * Whether we swapped r and od while enforcing frequency limits
         */
        let mut swapped: bool = false;
        let last_od: u32 = od;
        let last_r: u32 = r;

        /*
         * Try the next largest value for f (or r and od) and
         * recalculate the other parameters based on that
         */
        if freq_out > freq_in {
            /*
             * Skip factors of the same product if we already tried
             * out that product
             */
            while r * od == last_r * last_od {
                match factors.next() {
                    Some(factor) => {
                        r = unpack_r(&factor);
                        od = unpack_od(&factor);
                    },
                    None => break 'outer,
                }
            }

            /* Round close */
            f = ((((r * od) as u64) * ratio + bit(31)) >> 32) as u32;
            if f > F_MAX {
                f = F_MAX;
            }
        } else {
            f += 1;
            let tmp: u64 = (f as u64) * inv_ratio;
            let round_up: bool = tmp & bit(31) != 0;
            let goal: u32 = ((tmp >> 32) as u32) + (round_up as u32);
            let (err, last_err): (u32, u32);

            /*
             * Get the next r/od pair in factors. If the last_* pair is better,
             * then we will use it instead, so don't call next until after we're
             * sure we won't need this pair.
             */
            loop {
                match factors.peek() {
                    Some(factor) => {
                        r = unpack_r(factor);
                        od = unpack_od(factor)
                    },
                    None => break 'outer,
                }

                if r * od < goal {
                    factors.next();
                } else {
                    break;
                }
            }

            /*
             * This is a case of double rounding. If we rounded up
             * above, we need to round down (in cases of ties) here.
             * This prevents off-by-one errors resulting from
             * choosing X+2 over X when X.Y rounds up to X+1 and
             * there is no r * od = X+1. For the converse, when X.Y
             * is rounded down to X, we should choose X+1 over X-1.
             */
            err = abs_diff(r * od, goal);
            last_err = abs_diff(last_r * last_od, goal);
            if last_err < err || (round_up && last_err == err) {
                r = last_r;
                od = last_od;
            } else {
                factors.next();
            }
        }

        /*
         * Enforce limits on internal clock frequencies. If we
         * aren't in spec, try swapping r and od. If everything is
         * in-spec, calculate the relative error.
         */
        loop {
            /*
             * Whether the intermediate frequencies are out-of-spec
             */
            let mut out_of_spec: bool = false;

            if r > max_r {
                out_of_spec = true;
            } else {
                /*
                 * There is no way to only divide once; we need
                 * to examine the frequency with and without the
                 * effect of od.
                 */
                let vco: u64 = div_round_closest((freq_in as u64) * (f as u64), r);

                if vco > VCO_MAX || vco < VCO_MIN {
                    out_of_spec = true;
                }
            }

            if out_of_spec {
                if !swapped {
                    let tmp: u32 = r;

                    r = od;
                    od = tmp;
                    swapped = true;
                    continue;
                } else {
                    /*
                     * Try looking ahead to see if there are
                     * additional factors for the same
                     * product.
                     */
                    match factors.peek() {
                        Some(factor) => {
                            let new_r: u32 = unpack_r(factor);
                            let new_od: u32 = unpack_od(factor);

                            if r * od == new_r * new_od {
                                factors.next();
                                r = new_r;
                                od = new_od;
                                swapped = false;
                                continue;
                            } else {
                                break;
                            }
                        },
                        None => break 'outer,
                    }
                }
            }

            error = div_round_closest((f as u64) * inv_ratio, r * od) as i64;
            /* The lower 16 bits are spurious */
            error = i64::abs(error - (bit(32) as i64)) >> 16;

            if error < best_error {
                best_error = error;
                best = Some(Params {
                    clkr: (r - 1) as u8,
                    clkf: (f - 1) as u8,
                    clkod: (od - 1) as u8,
                    bwadj: (f - 1) as u8,
                });
            }
            break;
        }

        if error == 0 {
            break;
        }
    }

    return best;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brute_force_params(freq_in: u32, freq_out: u32) -> Option<Params> {
        let mut best: Option<Params> = None;
        let (mut error, mut best_error): (i64, i64);
        let (max_r, inv_ratio): (u32, u64);

        best_error = i64::max_value();
        max_r = u32::min(R_MAX, (freq_in / DIV_MIN) as u32);
        inv_ratio = div_round_closest((freq_in as u64) << 32, freq_out);

        /* Brute force it */
        for r in R_MIN..=max_r {
            for f in F_MIN..=F_MAX {
                for od in OD_MIN..=OD_MAX {
                    let vco: u64 = div_round_closest((freq_in as u64) * (f as u64), r);

                    if vco > VCO_MAX || vco < VCO_MIN {
                        continue;
                    }

                    error = div_round_closest((f as u64) * inv_ratio, r * od) as i64;
                    /* The lower 16 bits are spurious */
                    error = i64::abs(error - (bit(32) as i64)) >> 16;
                    if error < best_error {
                        best_error = error;
                        best = Some(Params {
                            clkr: (r - 1) as u8,
                            clkf: (f - 1) as u8,
                            clkod: (od - 1) as u8,
                            bwadj: (f - 1) as u8,
                        });
                    }
                }
            }
        }

        return best;
    }

    fn params_equal(_a: Option<Params>, _b: Option<Params>) -> bool {
        match (_a, _b) {
            (None, None) => true,
            (Some(_), None) => false,
            (None, Some(_)) => false,
            (Some(a), Some(b)) => {
                let ar: u32 = (a.clkr + 1) as u32;
                let af: u32 = (a.clkf + 1) as u32;
                let aod: u32 = (a.clkod + 1) as u32;
                let br: u32 = (b.clkr + 1) as u32;
                let bf: u32 = (b.clkf + 1) as u32;
                let bod: u32 = (b.clkod + 1) as u32;

                af * br * bod == bf * ar * aod
            }
        }
    }

    fn verify_compute_params(freq_in: u32, freq_out: u32) -> bool {
        params_equal(compute_params(freq_in, freq_out), brute_force_params(freq_in, freq_out))
    }

    #[test]
    fn test_compute_params() {
        assert_eq!(compute_params(26_000_000, 0), None);
        assert_eq!(compute_params(0, 390_000_000), None);
        assert_eq!(compute_params(26_000_000, 2_000_000_000), None);
        assert_eq!(compute_params(2_000_000_000, 390_000_000), None);
        assert_eq!(compute_params(20_000_000, 1_500_000_000), None);

        assert!(verify_compute_params(26_000_000, 1_500_000_000));
        assert!(verify_compute_params(26_000_000, 1_000_000_000));
        assert!(verify_compute_params(26_000_000, 800_000_000));
        assert!(verify_compute_params(26_000_000, 700_000_000));
        assert!(verify_compute_params(26_000_000, 300_000_000));
        assert!(verify_compute_params(26_000_000, 45_158_400));
        assert!(verify_compute_params(26_000_000, 27_000_000));
        assert!(verify_compute_params(27_000_000, 26_000_000));
        assert!(verify_compute_params(390_000_000, 26_000_000));
        assert!(verify_compute_params(390_000_000, 383_846_400));
    }
}