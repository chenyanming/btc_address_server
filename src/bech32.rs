/// Human-readable part and data part separator
pub const SEP: char = '1';

/// Encoding character set. Maps data value -> char
pub const CHARSET: [char; 32] = [
    'q', 'p', 'z', 'r', 'y', '9', 'x', '8', //  +0
    'g', 'f', '2', 't', 'v', 'd', 'w', '0', //  +8
    's', '3', 'j', 'n', '5', '4', 'k', 'h', // +16
    'c', 'e', '6', 'm', 'u', 'a', '7', 'l', // +24
];

/// Expand the HRP into values for checksum computation.
fn bech32_hrp_expand(hrp: &str) -> Vec<u8> {
    let mut v = vec![];
    v.extend(
        hrp.chars()
            .map(|x| (x as u32 >> 5) as u8)
            .collect::<Vec<u8>>(),
    );
    v.extend([0]);
    v.extend(
        hrp.chars()
            .map(|x| (x as u32 & 31) as u8)
            .collect::<Vec<u8>>(),
    );
    v
}

/// Internal function that computes the Bech32 checksum.
fn bech32_polymod(values: Vec<u8>) -> u32 {
    let generator: [u32; 5] = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
    let mut chk: u32 = 1;
    let mut top: u32 = 0;
    for value in values {
        top = chk >> 25;
        chk = (chk & 0x1ffffff) << 5 ^ (value as u32);
        for i in 0..5 {
            if (top >> i) & 1 == 1 {
                chk ^= generator[i];
            } else {
                chk ^= 0;
            }
        }
    }
    chk
}

/// Compute the checksum values given HRP and data.
pub fn bech32_create_checksum(hrp: &str, data: &Vec<u8>) -> Vec<u8> {
    let mut values = vec![];
    values.extend(bech32_hrp_expand(hrp));
    values.extend(data.to_owned());
    values.extend([0, 0, 0, 0, 0, 0]);
    let polymod = bech32_polymod(values) ^ 1;
    let mut checksum = vec![];
    for i in 0..6 {
        checksum.push(((polymod >> 5 * (5 - i)) & 31) as u8);
    }
    checksum
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_bech32_hrp_expand() {
        assert_eq!(bech32_hrp_expand("bc"), vec![3, 3, 0, 2, 3]);
    }

    #[test]
    fn test_bech32_polymod() {
        assert_eq!(bech32_polymod(vec![0, 1]), 1025);
        // let mut v = vec![];
        // v.extend_from_slice(&bech32_hrp_expand("bc"));
        // v.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
        // assert_eq!(bech32_polymod(v), 1025);
    }

    #[test]
    fn test_bech32_create_checksum() {
        assert_eq!(
            bech32_create_checksum("bc", vec![0, 1]),
            vec![13, 11, 10, 5, 18, 27]
        );
    }
}
