// Minimal read-only zip reader for a File/Blob. Reads only the central directory and the
// entries asked for, via Blob.slice(), so a multi-GB package is never loaded into memory.
// Supports ZIP64 (Discord's events files are > 4 GB uncompressed).

const EOCD = 0x06054b50;
const ZIP64_LOCATOR = 0x07064b50;
const ZIP64_EOCD = 0x06064b50;
const CENTRAL = 0x02014b50;
const LOCAL = 0x04034b50;

async function view(blob, start, end) {
  return new DataView(await blob.slice(start, end).arrayBuffer());
}

const u64 = (dv, o) => Number(dv.getBigUint64(o, true));

/** Lists entries: [{ name, method, compressedSize, size, localOffset, modified }]. */
export async function listEntries(blob) {
  const tailLen = Math.min(blob.size, 65557);
  const tail = await view(blob, blob.size - tailLen, blob.size);
  let eocd = -1;
  for (let i = tailLen - 22; i >= 0; i--) {
    if (tail.getUint32(i, true) === EOCD) { eocd = i; break; }
  }
  if (eocd < 0) throw new Error("This doesn't look like a zip file.");

  let count = tail.getUint16(eocd + 10, true);
  let cdSize = tail.getUint32(eocd + 12, true);
  let cdOffset = tail.getUint32(eocd + 16, true);
  if (eocd >= 20 && tail.getUint32(eocd - 20, true) === ZIP64_LOCATOR) {
    const z64 = await view(blob, u64(tail, eocd - 12), u64(tail, eocd - 12) + 56);
    if (z64.getUint32(0, true) !== ZIP64_EOCD) throw new Error("Broken ZIP64 header.");
    count = u64(z64, 32);
    cdSize = u64(z64, 40);
    cdOffset = u64(z64, 48);
  }

  const cd = await view(blob, cdOffset, cdOffset + cdSize);
  const utf8 = new TextDecoder();
  const entries = [];
  let p = 0;
  for (let n = 0; n < count; n++) {
    if (cd.getUint32(p, true) !== CENTRAL) throw new Error("Broken zip directory.");
    const method = cd.getUint16(p + 10, true);
    const modified = dosTime(cd.getUint16(p + 14, true), cd.getUint16(p + 12, true));
    let compressedSize = cd.getUint32(p + 20, true);
    let size = cd.getUint32(p + 24, true);
    const nameLen = cd.getUint16(p + 28, true);
    const extraLen = cd.getUint16(p + 30, true);
    const commentLen = cd.getUint16(p + 32, true);
    let localOffset = cd.getUint32(p + 42, true);
    const name = utf8.decode(new Uint8Array(cd.buffer, cd.byteOffset + p + 46, nameLen));

    // ZIP64 extra field: 64-bit values for whichever fields were maxed out, in this order.
    let e = p + 46 + nameLen;
    const extraEnd = e + extraLen;
    while (e + 4 <= extraEnd) {
      const id = cd.getUint16(e, true);
      const len = cd.getUint16(e + 2, true);
      if (id === 0x0001) {
        let q = e + 4;
        if (size === 0xffffffff) { size = u64(cd, q); q += 8; }
        if (compressedSize === 0xffffffff) { compressedSize = u64(cd, q); q += 8; }
        if (localOffset === 0xffffffff) { localOffset = u64(cd, q); }
      }
      e += 4 + len;
    }
    entries.push({ name, method, compressedSize, size, localOffset, modified });
    p = extraEnd + commentLen;
  }
  return entries;
}

/** MS-DOS date and time fields as Unix ms (the zip stores local time; treated as UTC). */
function dosTime(date, time) {
  if (!date) return 0;
  return Date.UTC(1980 + (date >> 9), ((date >> 5) & 15) - 1, date & 31, time >> 11, (time >> 5) & 63, (time & 31) * 2);
}

async function dataRange(blob, entry) {
  const h = await view(blob, entry.localOffset, entry.localOffset + 30);
  if (h.getUint32(0, true) !== LOCAL) throw new Error(`Broken entry: ${entry.name}`);
  const start = entry.localOffset + 30 + h.getUint16(26, true) + h.getUint16(28, true);
  return [start, start + entry.compressedSize];
}

/**
 * Streams an entry's uncompressed bytes.
 * `onCompressedBytes(n)` is called as compressed input is consumed (for progress bars).
 */
export async function openEntry(blob, entry, onCompressedBytes) {
  const [start, end] = await dataRange(blob, entry);
  let raw = blob.slice(start, end).stream();
  if (onCompressedBytes) {
    raw = raw.pipeThrough(new TransformStream({
      transform(chunk, ctl) { onCompressedBytes(chunk.byteLength); ctl.enqueue(chunk); },
    }));
  }
  if (entry.method === 0) return raw;
  if (entry.method === 8) return raw.pipeThrough(new DecompressionStream("deflate-raw"));
  throw new Error(`Unsupported compression (method ${entry.method}) in ${entry.name}`);
}

/** Reads a whole (small) entry into a Uint8Array. */
export async function readEntry(blob, entry) {
  if (entry.method === 0) {
    const [start, end] = await dataRange(blob, entry);
    return new Uint8Array(await blob.slice(start, end).arrayBuffer());
  }
  return new Uint8Array(await new Response(await openEntry(blob, entry)).arrayBuffer());
}
