import zlib from "zlib";
import fs from "fs";
import path from "path";

const crcTable = [];
for (let n = 0; n < 256; n++) {
  let c = n;
  for (let k = 0; k < 8; k++) {
    c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  }
  crcTable[n] = c;
}

function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc = crcTable[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, "ascii");
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

function createPng(width, height, drawFn) {
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  const stride = width * 4;
  const raw = Buffer.alloc(height * (stride + 1));
  for (let y = 0; y < height; y++) {
    raw[y * (stride + 1)] = 0;
    for (let x = 0; x < width; x++) {
      const [r, g, b, a] = drawFn(x, y, width, height);
      const off = y * (stride + 1) + 1 + x * 4;
      raw[off] = r;
      raw[off + 1] = g;
      raw[off + 2] = b;
      raw[off + 3] = a;
    }
  }

  const compressed = zlib.deflateSync(raw);
  return Buffer.concat([
    sig,
    chunk("IHDR", ihdr),
    chunk("IDAT", compressed),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

const size = 512;
const png = createPng(size, size, (x, y, w, h) => {
  const cx = w / 2;
  const cy = h / 2;
  const r = w * 0.38;
  const dx = x - cx;
  const dy = y - cy;
  const dist = Math.sqrt(dx * dx + dy * dy);

  if (dist < r * 0.55) {
    return [59, 130, 246, 255];
  }
  if (dist < r * 0.65) {
    return [30, 64, 128, 255];
  }
  if (dist < r) {
    const t = (dist - r * 0.65) / (r * 0.35);
    return [30, 30, 35, 255];
  }
  return [10, 10, 11, 255];
});

const outDir = path.resolve(import.meta.dirname, "..", "src-tauri", "icons");
fs.mkdirSync(outDir, { recursive: true });
fs.writeFileSync(path.join(outDir, "icon.png"), png);
console.log("Icon generated:", path.join(outDir, "icon.png"));
