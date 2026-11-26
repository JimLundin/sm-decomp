//! SHA-256, for the vectors' image digests.

/// The round constants.
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

/// The initial hash.
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

const BLOCK: usize = 64;
const LENGTH_AT: usize = 56;

/// The schedule of one message block.
fn schedule(block: &[u8]) -> [u32; 64] {
    let mut w = [0; 64];
    for (slot, bytes) in w.iter_mut().zip(block.chunks(4)) {
        *slot = u32::from_be_bytes(bytes.try_into().unwrap());
    }
    for i in 16..BLOCK {
        let early = w[i.wrapping_sub(15)];
        let late = w[i.wrapping_sub(2)];
        let s0 = early.rotate_right(7) ^ early.rotate_right(18) ^ (early >> 3);
        let s1 = late.rotate_right(17) ^ late.rotate_right(19) ^ (late >> 10);
        w[i] = w[i.wrapping_sub(16)]
            .wrapping_add(s0)
            .wrapping_add(w[i.wrapping_sub(7)])
            .wrapping_add(s1);
    }
    w
}

/// Add the next message block to the digest.
fn compress(hash: &mut [u32; 8], block: &[u8]) {
    let mut working = *hash;
    for (round, scheduled) in ROUND.iter().zip(schedule(block)) {
        let [first, second, third, fourth, fifth, sixth, seventh, eighth] = working;
        let s1 = fifth.rotate_right(6) ^ fifth.rotate_right(11) ^ fifth.rotate_right(25);
        let choice = (fifth & sixth) ^ (!fifth & seventh);
        let t1 = eighth
            .wrapping_add(s1)
            .wrapping_add(choice)
            .wrapping_add(*round)
            .wrapping_add(scheduled);
        let s0 = first.rotate_right(2) ^ first.rotate_right(13) ^ first.rotate_right(22);
        let majority = (first & second) ^ (first & third) ^ (second & third);
        let t2 = s0.wrapping_add(majority);
        working = [
            t1.wrapping_add(t2),
            first,
            second,
            third,
            fourth.wrapping_add(t1),
            fifth,
            sixth,
            seventh,
        ];
    }
    for (word, add) in hash.iter_mut().zip(working) {
        *word = word.wrapping_add(add);
    }
}

/// The SHA-256 digest of `data`, in lower-case hex.
pub fn hex_digest(data: &[u8]) -> String {
    let bits = u64::try_from(data.len()).unwrap().wrapping_mul(8);
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % BLOCK != LENGTH_AT {
        message.push(0);
    }
    message.extend(bits.to_be_bytes());
    let mut hash = INITIAL;
    for block in message.chunks(BLOCK) {
        compress(&mut hash, block);
    }
    hash.iter()
        .flat_map(|word| word.to_be_bytes())
        .fold(String::new(), |hex, byte| hex + &format!("{byte:02x}"))
}

#[test]
fn digest_of_abc_is_the_standard_one() {
    assert_eq!(
        hex_digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
