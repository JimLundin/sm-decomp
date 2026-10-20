use std::ffi::{c_char, c_long, c_void};
use std::mem::size_of;
use std::ptr::null_mut;
type stbi_uc = u8;
type stbi__uint16 = u16;
type stbi__uint32 = u32;
type size_t = usize;
use crate::c_library::{__assert_fail, free, malloc, memcpy, memset, realloc};
const UINT_MAX: u32 = u32::MAX;
unsafe fn stbi__malloc(size: size_t) -> *mut c_void {
    malloc(size)
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct stbi__zbuf {
        pub zbuffer, zbuffer_end: *mut stbi_uc;
        pub num_bits: i32;
        pub code_buffer: stbi__uint32;
        pub zout, zout_start, zout_end: *mut c_char;
        pub z_expandable: i32;
        pub z_length, z_distance: stbi__zhuffman;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stbi__zhuffman {
    pub fast: [stbi__uint16; 512],
    pub firstcode: [stbi__uint16; 16],
    pub maxcode: [i32; 17],
    pub firstsymbol: [stbi__uint16; 16],
    pub size: [stbi_uc; 288],
    pub value: [stbi__uint16; 288],
}
impl stbi__zhuffman {
    const EMPTY: Self = stbi__zhuffman {
        fast: [0; 512],
        firstcode: [0; 16],
        maxcode: [0; 17],
        firstsymbol: [0; 16],
        size: [0; 288],
        value: [0; 288],
    };
}
impl stbi__zbuf {
    const EMPTY: Self = stbi__zbuf {
        zbuffer: null_mut::<stbi_uc>(),
        zbuffer_end: null_mut::<stbi_uc>(),
        num_bits: 0,
        code_buffer: 0,
        zout: null_mut::<c_char>(),
        zout_start: null_mut::<c_char>(),
        zout_end: null_mut::<c_char>(),
        z_expandable: 0,
        z_length: stbi__zhuffman::EMPTY,
        z_distance: stbi__zhuffman::EMPTY,
    };
}
pub const STBI__ZFAST_BITS: i32 = 9;
pub const STBI__ZFAST_MASK: i32 = ((1i32) << STBI__ZFAST_BITS) - 1i32;
pub const STBI__ZNSYMS: i32 = 288;
unsafe fn stbi__bitreverse16(mut n: i32) -> i32 {
    n = (n & 0xaaaa) >> 1 | (n & 0x5555) << 1;
    n = (n & 0xcccc) >> 2 | (n & 0x3333) << 2;
    n = (n & 0xf0f0) >> 4 | (n & 0xf0f) << 4;
    n = (n & 0xff00) >> 8 | (n & 0xff) << 8;
    n
}
unsafe fn stbi__bit_reverse(v: i32, bits: i32) -> i32 {
    crate::native_assertion! {
        '_c2rust_label: bits <= 16;
        b"bits <= 16\0", b"./third_party/stb/stb_image.h\0", 4061,
        b"int stbi__bit_reverse(int, int)\0"
    };
    stbi__bitreverse16(v) >> 16 - bits
}
unsafe fn stbi__zbuild_huffman(z: *mut stbi__zhuffman, sizelist: *const stbi_uc, num: i32) -> i32 {
    let mut k: i32 = 0;
    let mut next_code: [i32; 16] = [0; 16];
    let mut sizes: [i32; 17] = [0; 17];
    memset(
        &raw mut sizes as *mut i32 as *mut c_void,
        0,
        size_of::<[i32; 17]>() as size_t,
    );
    memset(
        &raw mut (*z).fast as *mut stbi__uint16 as *mut c_void,
        0,
        size_of::<[stbi__uint16; 512]>() as size_t,
    );
    let mut i: i32 = 0;
    while i < num {
        sizes[*sizelist.offset(i as isize) as usize] += 1;
        i += 1;
    }
    sizes[0i32 as usize] = 0;
    for i in 1i32..16 {
        if sizes[i as usize] > (1i32) << i {
            return 0;
        }
    }
    let mut code: i32 = 0;
    for i in 1i32..16 {
        next_code[i as usize] = code;
        (*z).firstcode[i as usize] = code as stbi__uint16;
        (*z).firstsymbol[i as usize] = k as stbi__uint16;
        code = code + sizes[i as usize];
        if sizes[i as usize] != 0 && code - 1i32 >= (1i32) << i {
            return 0;
        }
        (*z).maxcode[i as usize] = code << 16 - i;
        code <<= 1;
        k += sizes[i as usize];
    }
    (*z).maxcode[16i32 as usize] = 0x10000;
    i = 0;
    while i < num {
        let s: i32 = *sizelist.offset(i as isize) as i32;
        if s != 0 {
            let c: i32 = next_code[s as usize] - (*z).firstcode[s as usize] as i32
                + (*z).firstsymbol[s as usize] as i32;
            let fastv: stbi__uint16 = (s << 9 | i) as stbi__uint16;
            (*z).size[c as usize] = s as stbi_uc;
            (*z).value[c as usize] = i as stbi__uint16;
            if s <= STBI__ZFAST_BITS {
                let mut j: i32 = stbi__bit_reverse(next_code[s as usize], s);
                while j < (1i32) << STBI__ZFAST_BITS {
                    (*z).fast[j as usize] = fastv;
                    j += (1i32) << s;
                }
            }
            next_code[s as usize] += 1;
        }
        i += 1;
    }
    1
}
unsafe fn stbi__zeof(z: *mut stbi__zbuf) -> i32 {
    ((*z).zbuffer >= (*z).zbuffer_end) as i32
}
unsafe fn stbi__zget8(z: *mut stbi__zbuf) -> stbi_uc {
    ((if stbi__zeof(z) != 0 {
        0
    } else {
        crate::update_place! { previous fresh49 = (*z).zbuffer => (*z).zbuffer.offset(1); }
        *fresh49 as i32
    }) as stbi_uc)
}
unsafe fn stbi__fill_bits(z: *mut stbi__zbuf) {
    do_until! { ((*z).num_bits > 24);
        if (*z).code_buffer >= (1 as stbi__uint32) << (*z).num_bits {
            (*z).zbuffer = (*z).zbuffer_end;
            return;
        }
        (*z).code_buffer =
            ((*z).code_buffer | (stbi__zget8(z) as u32) << (*z).num_bits) as stbi__uint32;
        (*z).num_bits += 8;
    }
}
unsafe fn stbi__zreceive(z: *mut stbi__zbuf, n: i32) -> u32 {
    if (*z).num_bits < n {
        stbi__fill_bits(z);
    }
    let mut k: u32 = ((*z).code_buffer & (((1i32) << n) - 1i32) as stbi__uint32);
    (*z).code_buffer >>= n;
    (*z).num_bits -= n;
    k
}
unsafe fn stbi__zhuffman_decode_slowpath(a: *mut stbi__zbuf, z: *mut stbi__zhuffman) -> i32 {
    let mut k: i32 = stbi__bit_reverse((*a).code_buffer as i32, 16);
    let mut s: i32 = STBI__ZFAST_BITS + 1;
    while (k >= (*z).maxcode[s as usize]) {
        s += 1;
    }
    if s >= 16 {
        return -1i32;
    }
    let mut b: i32 =
        (k >> 16 - s) - (*z).firstcode[s as usize] as i32 + (*z).firstsymbol[s as usize] as i32;
    if b >= STBI__ZNSYMS {
        return -1i32;
    }
    if (*z).size[b as usize] as i32 != s {
        return -1i32;
    }
    (*a).code_buffer >>= s;
    (*a).num_bits -= s;
    (*z).value[b as usize] as i32
}
unsafe fn stbi__zhuffman_decode(a: *mut stbi__zbuf, z: *mut stbi__zhuffman) -> i32 {
    if (*a).num_bits < 16 {
        if stbi__zeof(a) != 0 {
            return -1i32;
        }
        stbi__fill_bits(a);
    }
    let mut b: i32 =
        (*z).fast[((*a).code_buffer & STBI__ZFAST_MASK as stbi__uint32) as usize] as i32;
    if b != 0 {
        let mut s: i32 = b >> 9;
        (*a).code_buffer >>= s;
        (*a).num_bits -= s;
        return b & 511;
    }
    stbi__zhuffman_decode_slowpath(a, z)
}
unsafe fn stbi__zexpand(z: *mut stbi__zbuf, zout: *mut c_char, n: i32) -> i32 {
    (*z).zout = zout;
    if (*z).z_expandable == 0 {
        return 0;
    }
    let mut cur: u32 = (*z).zout.offset_from((*z).zout_start) as c_long as u32;
    let mut old_limit: u32 = (*z).zout_end.offset_from((*z).zout_start) as c_long as u32;
    let mut limit: u32 = old_limit;
    if UINT_MAX.wrapping_sub(cur) < n as u32 {
        return 0;
    }
    while cur.wrapping_add(n as u32) > limit {
        if limit > UINT_MAX.wrapping_div(2) {
            return 0;
        }
        limit = limit.wrapping_mul(2);
    }
    let mut q = realloc((*z).zout_start as *mut c_void, limit as size_t) as *mut c_char;
    if q.is_null() {
        return 0;
    }
    (*z).zout_start = q;
    (*z).zout = q.offset(cur as isize);
    (*z).zout_end = q.offset(limit as isize);
    1
}
static mut stbi__zlength_base: [i32; 31] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258, 0, 0,
];
static mut stbi__zlength_extra: [i32; 31] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 0, 0,
];
static mut stbi__zdist_base: [i32; 32] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577, 0, 0,
];
static mut stbi__zdist_extra: [i32; 32] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 0, 0,
];
unsafe fn stbi__parse_huffman_block(a: *mut stbi__zbuf) -> i32 {
    let mut zout = (*a).zout;
    loop {
        let mut z: i32 = stbi__zhuffman_decode(a, &raw mut (*a).z_length);
        if z < 256 {
            if z < 0 {
                return 0;
            }
            if zout >= (*a).zout_end {
                if stbi__zexpand(a, zout, 1) == 0 {
                    return 0;
                }
                zout = (*a).zout;
            }
            crate::update_place! { previous fresh45 = zout => zout.offset(1); }
            *fresh45 = z as c_char;
        } else {
            if z == 256 {
                (*a).zout = zout;
                return 1;
            }
            z -= 257;
            let mut len: i32 = stbi__zlength_base[z as usize];
            if stbi__zlength_extra[z as usize] != 0 {
                len = (len as u32).wrapping_add(stbi__zreceive(a, stbi__zlength_extra[z as usize]))
                    as i32;
            }
            z = stbi__zhuffman_decode(a, &raw mut (*a).z_distance);
            if z < 0 {
                return 0;
            }
            let mut dist: i32 = stbi__zdist_base[z as usize];
            if stbi__zdist_extra[z as usize] != 0 {
                dist = (dist as u32).wrapping_add(stbi__zreceive(a, stbi__zdist_extra[z as usize]))
                    as i32;
            }
            if (zout.offset_from((*a).zout_start) as c_long) < dist as c_long {
                return 0;
            }
            if zout.offset(len as isize) > (*a).zout_end {
                if stbi__zexpand(a, zout, len) == 0 {
                    return 0;
                }
                zout = (*a).zout;
            }
            let mut p = zout.offset(-(dist as isize)) as *mut stbi_uc;
            if dist == 1 {
                let v: stbi_uc = *p;
                if len != 0 {
                    do_until! { (len == 0);
                        crate::update_place! { previous fresh46 = zout => zout.offset(1); }
                        *fresh46 = v as c_char;
                        len -= 1;
                    }
                }
            } else if len != 0 {
                do_until! { (len == 0);
                    crate::update_place! { previous fresh47 = p => p.offset(1); }
                    crate::update_place! { previous fresh48 = zout => zout.offset(1); }
                    *fresh48 = *fresh47 as c_char;
                    len -= 1;
                }
            }
        }
    }
}
unsafe fn stbi__compute_huffman_codes(a: *mut stbi__zbuf) -> i32 {
    static mut length_dezigzag: [stbi_uc; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut z_codelength: stbi__zhuffman = stbi__zhuffman::EMPTY;
    let mut lencodes: [stbi_uc; 455] = [0; 455];
    let mut codelength_sizes: [stbi_uc; 19] = [0; 19];
    let hlit: i32 = stbi__zreceive(a, 5i32).wrapping_add(257) as i32;
    let hdist: i32 = stbi__zreceive(a, 5i32).wrapping_add(1) as i32;
    let hclen: i32 = stbi__zreceive(a, 4i32).wrapping_add(4) as i32;
    let ntot: i32 = hlit + hdist;
    memset(
        &raw mut codelength_sizes as *mut stbi_uc as *mut c_void,
        0,
        size_of::<[stbi_uc; 19]>() as size_t,
    );
    let mut i: i32 = 0;
    while i < hclen {
        let s: i32 = stbi__zreceive(a, 3) as i32;
        codelength_sizes[length_dezigzag[i as usize] as usize] = s as stbi_uc;
        i += 1;
    }
    if stbi__zbuild_huffman(
        &raw mut z_codelength,
        &raw mut codelength_sizes as *mut stbi_uc,
        19,
    ) == 0
    {
        return 0;
    }
    let mut n: i32 = 0;
    while n < ntot {
        let mut c: i32 = stbi__zhuffman_decode(a, &raw mut z_codelength);
        if c < 0 || c >= 19 {
            return 0;
        }
        if c < 16 {
            crate::update_place! { previous fresh50 = n => n + 1; }
            lencodes[fresh50 as usize] = c as stbi_uc;
        } else {
            let mut fill: stbi_uc = 0 as stbi_uc;
            if c == 16 {
                c = stbi__zreceive(a, 2i32).wrapping_add(3) as i32;
                if n == 0 {
                    return 0;
                }
                fill = lencodes[(n - 1i32) as usize];
            } else if c == 17 {
                c = stbi__zreceive(a, 3i32).wrapping_add(3) as i32;
            } else if c == 18 {
                c = stbi__zreceive(a, 7i32).wrapping_add(11) as i32;
            } else {
                return 0;
            }
            if ntot - n < c {
                return 0;
            }
            memset(
                (&raw mut lencodes as *mut stbi_uc).offset(n as isize) as *mut c_void,
                fill as i32,
                c as size_t,
            );
            n += c;
        }
    }
    if n != ntot {
        return 0;
    }
    if stbi__zbuild_huffman(
        &raw mut (*a).z_length,
        &raw mut lencodes as *mut stbi_uc,
        hlit,
    ) == 0
    {
        return 0;
    }
    if stbi__zbuild_huffman(
        &raw mut (*a).z_distance,
        (&raw mut lencodes as *mut stbi_uc).offset(hlit as isize),
        hdist,
    ) == 0
    {
        return 0;
    }
    1
}
unsafe fn stbi__parse_uncompressed_block(a: *mut stbi__zbuf) -> i32 {
    let mut header: [stbi_uc; 4] = [0; 4];
    if (*a).num_bits & 7 != 0 {
        stbi__zreceive(a, (*a).num_bits & 7);
    }
    let mut k: i32 = 0;
    while (*a).num_bits > 0 {
        crate::update_place! { previous fresh51 = k => k + 1; }
        header[fresh51 as usize] = ((*a).code_buffer & 255 as stbi__uint32) as stbi_uc;
        (*a).code_buffer >>= 8;
        (*a).num_bits -= 8;
    }
    if (*a).num_bits < 0 {
        return 0;
    }
    while k < 4 {
        crate::update_place! { previous fresh52 = k => k + 1; }
        header[fresh52 as usize] = stbi__zget8(a);
    }
    let mut len: i32 = header[1i32 as usize] as i32 * 256 + header[0i32 as usize] as i32;
    let mut nlen: i32 = header[3i32 as usize] as i32 * 256 + header[2i32 as usize] as i32;
    if nlen != len ^ 0xffff {
        return 0;
    }
    if (*a).zbuffer.offset(len as isize) > (*a).zbuffer_end {
        return 0;
    }
    if (*a).zout.offset(len as isize) > (*a).zout_end && stbi__zexpand(a, (*a).zout, len) == 0 {
        return 0;
    }
    memcpy(
        (*a).zout as *mut c_void,
        (*a).zbuffer as *const c_void,
        len as size_t,
    );
    (*a).zbuffer = (*a).zbuffer.offset(len as isize);
    (*a).zout = (*a).zout.offset(len as isize);
    1
}
unsafe fn stbi__parse_zlib_header(a: *mut stbi__zbuf) -> i32 {
    let cmf: i32 = stbi__zget8(a) as i32;
    let cm: i32 = cmf & 15;
    let flg: i32 = stbi__zget8(a) as i32;
    if stbi__zeof(a) != 0 {
        return 0;
    }
    if (cmf * 256 + flg) % 31 != 0 {
        return 0;
    }
    if flg & 32 != 0 {
        return 0;
    }
    if cm != 8 {
        return 0;
    }
    1
}
static mut stbi__zdefault_length: [stbi_uc; 288] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
];
static mut stbi__zdefault_distance: [stbi_uc; 32] = [
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
];
unsafe fn stbi__parse_zlib(a: *mut stbi__zbuf, parse_header: i32) -> i32 {
    if parse_header != 0 && stbi__parse_zlib_header(a) == 0 {
        return 0;
    }
    (*a).num_bits = 0;
    (*a).code_buffer = 0 as stbi__uint32;
    do_until! { (final_0 != 0);
        let mut final_0: i32 = stbi__zreceive(a, 1) as i32;
        let mut type_0: i32 = stbi__zreceive(a, 2) as i32;
        if type_0 == 0 {
            if stbi__parse_uncompressed_block(a) == 0 {
                return 0;
            }
        } else if type_0 == 3 {
            return 0;
        } else {
            if type_0 == 1 {
                if stbi__zbuild_huffman(
                    &raw mut (*a).z_length,
                    &raw const stbi__zdefault_length as *const stbi_uc,
                    STBI__ZNSYMS,
                ) == 0
                {
                    return 0;
                }
                if stbi__zbuild_huffman(
                    &raw mut (*a).z_distance,
                    &raw const stbi__zdefault_distance as *const stbi_uc,
                    32,
                ) == 0
                {
                    return 0;
                }
            } else if stbi__compute_huffman_codes(a) == 0 {
                return 0;
            }
            if stbi__parse_huffman_block(a) == 0 {
                return 0;
            }
        }
    }
    1
}
unsafe fn stbi__do_zlib(
    a: *mut stbi__zbuf,
    obuf: *mut c_char,
    olen: i32,
    exp: i32,
    parse_header: i32,
) -> i32 {
    (*a).zout_start = obuf;
    (*a).zout = obuf;
    (*a).zout_end = obuf.offset(olen as isize);
    (*a).z_expandable = exp;
    stbi__parse_zlib(a, parse_header)
}
pub unsafe fn stbi_zlib_decode_malloc_guesssize_headerflag(
    buffer: *const c_char,
    len: i32,
    initial_size: i32,
    outlen: *mut i32,
    parse_header: i32,
) -> *mut c_char {
    let mut a: stbi__zbuf = stbi__zbuf::EMPTY;
    let p = stbi__malloc(initial_size as size_t) as *mut c_char;
    if p.is_null() {
        return null_mut::<c_char>();
    }
    a.zbuffer = buffer as *mut stbi_uc;
    a.zbuffer_end = (buffer as *mut stbi_uc).offset(len as isize);
    if stbi__do_zlib(&raw mut a, p, initial_size, 1, parse_header) != 0 {
        if !outlen.is_null() {
            *outlen = a.zout.offset_from(a.zout_start) as c_long as i32;
        }
        a.zout_start
    } else {
        free(a.zout_start as *mut c_void);
        null_mut::<c_char>()
    }
}

pub(super) fn inflate(input: &[u8], initial_size: usize, header: bool) -> Option<Vec<u8>> {
    unsafe {
        let mut size = 0;
        let data = stbi_zlib_decode_malloc_guesssize_headerflag(
            input.as_ptr().cast(),
            input.len().try_into().ok()?,
            initial_size.try_into().ok()?,
            &mut size,
            i32::from(header),
        );
        if data.is_null() {
            return None;
        }
        let result = std::slice::from_raw_parts(data.cast::<u8>(), size as usize).to_vec();
        free(data.cast());
        Some(result)
    }
}
