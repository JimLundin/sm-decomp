use crate::canonical::{Reader, varint};
use crate::{BLOB, IMAGE, Id, NODE, Store};
use anyhow::{Result, ensure};
use std::io::Write;

const PAGE: usize = 256;
const FANOUT: usize = 16;
const MAX_IMAGE: u64 = 1 << 30;

pub fn put_image(store: &mut Store, bytes: &[u8]) -> Result<Id> {
    ensure!(bytes.len() as u64 <= MAX_IMAGE, "image exceeds limit");
    let mut level = bytes.chunks(PAGE).map(|p| store.put(BLOB, p)).collect::<Result<Vec<_>>>()?;
    let mut height = 0;
    while level.len() > 1 {
        height += 1;
        level = level.chunks(FANOUT).map(|c| store.put(NODE, &c.concat())).collect::<Result<_>>()?;
    }
    let mut out = Vec::new();
    varint(&mut out, bytes.len() as u64);
    varint(&mut out, height);
    if let Some(root) = level.first() {
        out.extend_from_slice(root);
    }
    store.put(IMAGE, &out)
}

fn header(store: &Store, id: &Id) -> Result<(u64, u32, Option<Id>)> {
    let bytes = store.get_kind(id, IMAGE)?;
    let mut r = Reader::new(&bytes);
    let len = r.varint()?;
    ensure!(len <= MAX_IMAGE, "image exceeds limit");
    let height = r.varint()?;
    let mut n = len.div_ceil(PAGE as u64);
    let mut expected = 0;
    while n > 1 {
        expected += 1;
        n = n.div_ceil(FANOUT as u64);
    }
    ensure!(height == expected, "invalid image height");
    let root = if len == 0 { None } else { Some(r.id()?) };
    r.done()?;
    Ok((len, height as u32, root))
}

fn children(store: &Store, id: &Id, expected: usize) -> Result<Vec<Id>> {
    let bytes = store.get_kind(id, NODE)?;
    ensure!(bytes.len() == expected * 32 && expected > 0 && expected <= FANOUT, "invalid image node shape");
    Ok(bytes.as_chunks::<32>().0.to_vec())
}

pub fn write_image(store: &Store, id: &Id, out: &mut dyn Write) -> Result<()> {
    let (len, height, root) = header(store, id)?;
    let mut work = root.map(|root| vec![(root, height, len)]).unwrap_or_default();
    while let Some((id, height, len)) = work.pop() {
        if height == 0 {
            let bytes = store.get_kind(&id, BLOB)?;
            ensure!(bytes.len() as u64 == len && len <= PAGE as u64, "image page length differs");
            out.write_all(&bytes)?;
        } else {
            let span = PAGE as u64 * (FANOUT as u64).pow(height - 1);
            for (i, child) in children(store, &id, len.div_ceil(span) as usize)?.into_iter().enumerate().rev() {
                work.push((child, height - 1, (len - i as u64 * span).min(span)));
            }
        }
    }
    Ok(())
}

/// Differing 256-byte page offsets, descending only through unequal subtrees.
/// The second result is the number of node objects read. Authenticate both
/// images with `write_image` before trusting equal roots from an external source.
pub fn diff_images(store: &Store, left: &Id, right: &Id) -> Result<(Vec<u64>, u64)> {
    let (len, height, lroot) = header(store, left)?;
    let (rlen, rheight, rroot) = header(store, right)?;
    ensure!(len == rlen && height == rheight, "images have different shapes");
    let mut work = match (lroot, rroot) {
        (Some(l), Some(r)) => vec![(l, r, height, 0, len)],
        _ => Vec::new(),
    };
    let mut offsets = Vec::new();
    let mut reads = 0;
    while let Some((l, r, height, offset, len)) = work.pop() {
        if l == r {
            continue;
        }
        if height == 0 {
            offsets.push(offset);
            continue;
        }
        let span = PAGE as u64 * (FANOUT as u64).pow(height - 1);
        let count = len.div_ceil(span) as usize;
        let lc = children(store, &l, count)?;
        let rc = children(store, &r, count)?;
        reads += 2;
        for i in (0..count).rev() {
            work.push((lc[i], rc[i], height - 1, offset + i as u64 * span, (len - i as u64 * span).min(span)));
        }
    }
    Ok((offsets, reads))
}
