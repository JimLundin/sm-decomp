//! SHA-256 (FIPS 180-4), to identify the ROM image without a registry dependency.

/// The first 32 bits of the fractional parts of the cube roots of the first 64 primes.
const ROUND: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// The first 32 bits of the fractional parts of the square roots of the first 8 primes.
const INITIAL: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// The SHA-256 digest of `message`.
#[must_use]
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut padded = message.to_vec();
    padded.push(0x80);
    // Zeros up to 8 bytes short of a block boundary, where the bit length goes.
    let padded_to = padded
        .len()
        .wrapping_add(8)
        .next_multiple_of(64)
        .wrapping_sub(8);
    padded.resize(padded_to, 0);
    let bits = u64::try_from(message.len())
        .unwrap_or(u64::MAX)
        .wrapping_mul(8);
    padded.extend_from_slice(&bits.to_be_bytes());
    let mut hash = INITIAL;
    for block in padded.as_chunks::<64>().0 {
        compress(&mut hash, block);
    }
    let mut digest = [0; 32];
    for (bytes, word) in digest.as_chunks_mut::<4>().0.iter_mut().zip(hash) {
        *bytes = word.to_be_bytes();
    }
    digest
}

/// One block's rounds. The window holds the schedule's next 16 words, so each round
/// consumes its first and appends the word 16 rounds on.
fn compress(hash: &mut [u32; 8], block: &[u8; 64]) {
    let mut window = [0; 16];
    for (word, bytes) in window.iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    let mut working = *hash;
    for &k in &ROUND {
        let [w0, w1, _, _, _, _, _, _, _, w9, _, _, _, _, w14, _] = window;
        let s0 = w1.rotate_right(7) ^ w1.rotate_right(18) ^ (w1 >> 3);
        let s1 = w14.rotate_right(17) ^ w14.rotate_right(19) ^ (w14 >> 10);
        window.rotate_left(1);
        window[15] = w0.wrapping_add(s0).wrapping_add(w9).wrapping_add(s1);

        let [va, vb, vc, vd, ve, vf, vg, vh] = working;
        let sum1 = ve.rotate_right(6) ^ ve.rotate_right(11) ^ ve.rotate_right(25);
        let choice = (ve & vf) ^ (!ve & vg);
        let t1 = vh
            .wrapping_add(sum1)
            .wrapping_add(choice)
            .wrapping_add(k)
            .wrapping_add(w0);
        let sum0 = va.rotate_right(2) ^ va.rotate_right(13) ^ va.rotate_right(22);
        let majority = (va & vb) ^ (va & vc) ^ (vb & vc);
        let t2 = sum0.wrapping_add(majority);
        working = [
            t1.wrapping_add(t2),
            va,
            vb,
            vc,
            vd.wrapping_add(t1),
            ve,
            vf,
            vg,
        ];
    }
    for (word, add) in hash.iter_mut().zip(working) {
        *word = word.wrapping_add(add);
    }
}
