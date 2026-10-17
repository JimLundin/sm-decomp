pub fn independent_adapter_vectors() {
    use crate::{Digest, Sha256};
    let vectors = [
        (0, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        (1, "084fed08b978af4d7d196a7446a86b58009e636b611db16211b65a9aadff29c5"),
        (65535, "4c32fbc9c2bad24dbe35c62f9aafd3e7dd6d2419f0b4b0a064db2fd79a56b6ab"),
        (65536, "f779dbe3aeaa609beb32212579f3aea441f244621d13c977cb8b52d7ec3016d9"),
        (65537, "5ea07e7659785552529c03843e59b785822a318d92800a44ed2aa1dc4df61400"),
        (131089, "ee4c95c36dbe4e78f0146daf77eb9feaa14f13cc173cab30cc4824724101f75e"),
    ];
    for (size, expected) in vectors {
        let bytes: Vec<u8> = (0..size).map(|i| ((17 * i + 3) % 256) as u8).collect();
        let mut expected_bytes = [0_u8; 32];
        for (i, pair) in expected.as_bytes().chunks_exact(2).enumerate() {
            let nibble = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => panic!("invalid independent vector"),
            };
            expected_bytes[i] = nibble(pair[0]) * 16 + nibble(pair[1]);
        }
        let direct = Sha256::digest(&bytes);
        assert_eq!(<[u8; 32]>::from(direct), expected_bytes);
        assert_eq!(direct.as_ref(), expected_bytes.as_slice());
        assert_eq!(&*direct, &expected_bytes);
        assert_eq!(format!("{direct:x}"), expected);
        let mut streaming = Sha256::new();
        for chunk in bytes.chunks(65536) {
            streaming.update(chunk);
        }
        assert_eq!(<[u8; 32]>::from(streaming.finalize()), expected_bytes);
        let split = bytes.len() / 2;
        let mut first = Sha256::new();
        first.update(&bytes[..split]);
        let mut cloned = first.clone();
        first.update(&bytes[split..]);
        cloned.update(&bytes[split..]);
        assert_eq!(<[u8; 32]>::from(first.finalize()), expected_bytes);
        assert_eq!(<[u8; 32]>::from(cloned.finalize()), expected_bytes);
    }
}
