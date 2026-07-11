#!/usr/bin/env node
/**
 * 生成 Peregrine 高清图标资源（icon.ico + icon.png）。
 *
 * 4x4 超采样抗锯齿，蓝色圆角背景 + 白色 "P" 字形。
 * ICO 包含 16/32/48/64/128/256 六档；PNG 为 512x512。
 * 无外部依赖，仅使用 Node.js 标准库。
 */

const fs = require("fs");
const path = require("path");
const zlib = require("zlib");

// "P" 字形位图（5 列 × 7 行）。
const GLYPH_P = [
  [1, 1, 1, 1, 0],
  [1, 0, 0, 0, 1],
  [1, 0, 0, 0, 1],
  [1, 1, 1, 1, 0],
  [1, 0, 0, 0, 0],
  [1, 0, 0, 0, 0],
  [1, 0, 0, 0, 0],
];

const BLUE = [37, 99, 235];
const WHITE = [255, 255, 255];

/** 点 (px,py) 是否在圆角矩形内。 */
function insideRoundedRect(px, py, x, y, w, h, r) {
  r = Math.max(0, Math.min(r, w / 2, h / 2));
  if (px < x || py < y || px >= x + w || py >= y + h) return false;
  const cx = Math.max(x + r, Math.min(px, x + w - 1 - r));
  const cy = Math.max(y + r, Math.min(py, y + h - 1 - r));
  const dx = px - cx;
  const dy = py - cy;
  return dx * dx + dy * dy <= r * r;
}

/** 返回 "P" 字形在坐标 的覆盖比例 [0,1]。 */
function glyphAlpha(px, py, gx0, gy0, gw, gh) {
  const gx = px - gx0;
  const gy = py - gy0;
  if (gx < 0 || gx >= gw || gy < 0 || gy >= gh) return 0;
  const cellW = gw / 5;
  const cellH = gh / 7;
  const col = Math.floor(gx / cellW);
  const row = Math.floor(gy / cellH);
  if (row >= 0 && row < 7 && col >= 0 && col < 5 && GLYPH_P[row][col] === 1) return 1;
  return 0;
}

/**
 * 渲染指定尺寸为 top-down RGBA 像素 Buffer，使用超采样抗锯齿。
 * @param {number} size
 * @param {number} samples 每边子采样数
 * @returns {Buffer} size*size*4 字节，top-down RGBA
 */
function renderIconTopDown(size, samples = 4) {
  const buf = Buffer.alloc(size * size * 4);
  const margin = size * 0.1;
  const sqMin = margin;
  const sqSize = Math.max(1, size - margin * 2);
  const corner = sqSize * 0.22;
  const pad = sqSize * 0.18;
  const gMinX = sqMin + pad;
  const gMinY = sqMin + pad;
  const gW = sqSize - pad * 2;
  const gH = sqSize - pad * 2;
  const invTotal = 1 / (samples * samples);

  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let rAcc = 0, gAcc = 0, bAcc = 0, aAcc = 0;
      for (let sy = 0; sy < samples; sy++) {
        for (let sx = 0; sx < samples; sx++) {
          const px = x + (sx + 0.5) / samples;
          const py = y + (sy + 0.5) / samples;
          if (!insideRoundedRect(px, py, sqMin, sqMin, sqSize, sqSize, corner)) continue;
          const ga = glyphAlpha(px, py, gMinX, gMinY, gW, gH);
          if (ga > 0.5) { rAcc += WHITE[0]; gAcc += WHITE[1]; bAcc += WHITE[2]; }
          else { rAcc += BLUE[0]; gAcc += BLUE[1]; bAcc += BLUE[2]; }
          aAcc += 255;
        }
      }
      const idx = (y * size + x) * 4;
      buf[idx] = Math.round(rAcc * invTotal);
      buf[idx + 1] = Math.round(gAcc * invTotal);
      buf[idx + 2] = Math.round(bAcc * invTotal);
      buf[idx + 3] = Math.round(aAcc * invTotal);
    }
  }
  return buf;
}

/** top-down RGBA → bottom-up BGRA（供 ICO BMP 使用）。 */
function toBottomUpBGRA(topDownRGBA, size) {
  const buf = Buffer.alloc(size * size * 4);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const src = (y * size + x) * 4;
      const dst = ((size - 1 - y) * size + x) * 4;
      buf[dst] = topDownRGBA[src + 2];     // B
      buf[dst + 1] = topDownRGBA[src + 1]; // G
      buf[dst + 2] = topDownRGBA[src];     // R
      buf[dst + 3] = topDownRGBA[src + 3]; // A
    }
  }
  return buf;
}

function buildBmpDib(width, height, bgraPixels) {
  // BITMAPINFOHEADER
  const header = Buffer.alloc(40);
  header.writeUInt32LE(40, 0);       // biSize
  header.writeInt32LE(width, 4);     // biWidth
  header.writeInt32LE(height * 2, 8);// biHeight (XOR + AND)
  header.writeUInt16LE(1, 12);       // biPlanes
  header.writeUInt16LE(32, 14);      // biBitCount
  // 其余字段为 0
  const rowBytes = Math.floor((width + 31) / 32) * 4;
  const andMask = Buffer.alloc(rowBytes * height);
  return Buffer.concat([header, bgraPixels, andMask]);
}

function buildICO(sizes, samples = 4) {
  const count = sizes.length;
  const icondir = Buffer.alloc(6);
  icondir.writeUInt16LE(0, 0); // reserved
  icondir.writeUInt16LE(1, 2); // type = icon
  icondir.writeUInt16LE(count, 4);

  const entries = [];
  const images = [];
  let offset = 6 + 16 * count;

  for (const size of sizes) {
    const rgba = renderIconTopDown(size, samples);
    const bgra = toBottomUpBGRA(rgba, size);
    const dib = buildBmpDib(size, size, bgra);
    const entry = Buffer.alloc(16);
    entry.writeUInt8(size < 256 ? size : 0, 0);  // width
    entry.writeUInt8(size < 256 ? size : 0, 1);  // height
    entry.writeUInt8(0, 2);  // color count
    entry.writeUInt8(0, 3);  // reserved
    entry.writeUInt16LE(1, 4);  // planes
    entry.writeUInt16LE(32, 6); // bit count
    entry.writeUInt32LE(dib.length, 8);  // size
    entry.writeUInt32LE(offset, 12);     // offset
    entries.push(entry);
    images.push(dib);
    offset += dib.length;
  }

  return Buffer.concat([icondir, ...entries, ...images]);
}

/** CRC32 查找表。 */
const crcTable = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    t[n] = c;
  }
  return t;
})();

function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc = crcTable[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function pngChunk(type, data) {
  const typeData = Buffer.concat([Buffer.from(type), data]);
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typeData), 0);
  return Buffer.concat([len, typeData, crc]);
}

function buildPNG(size, samples = 4) {
  const rgba = renderIconTopDown(size, samples);
  // 构建 raw 数据：每行前面加 filter byte (0)。
  const rows = [];
  for (let y = 0; y < size; y++) {
    rows.push(Buffer.from([0])); // filter: None
    rows.push(rgba.subarray(y * size * 4, (y + 1) * size * 4));
  }
  const raw = Buffer.concat(rows);
  const compressed = zlib.deflateSync(raw, { level: 9 });

  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);   // width
  ihdr.writeUInt32BE(size, 4);   // height
  ihdr.writeUInt8(8, 8);         // bit depth
  ihdr.writeUInt8(6, 9);         // color type: RGBA
  ihdr.writeUInt8(0, 10);        // compression
  ihdr.writeUInt8(0, 11);        // filter
  ihdr.writeUInt8(0, 12);        // interlace

  return Buffer.concat([
    signature,
    pngChunk("IHDR", ihdr),
    pngChunk("IDAT", compressed),
    pngChunk("IEND", Buffer.alloc(0)),
  ]);
}

// ===== 主入口 =====
const sizes = [16, 32, 48, 64, 128, 256];
const dir = __dirname;

const icoData = buildICO(sizes, 8);
const icoPath = path.join(dir, "icon.ico");
fs.writeFileSync(icoPath, icoData);
console.log(`generated ${icoPath} (${icoData.length} bytes)`);

const pngData = buildPNG(1024, 4);
const pngPath = path.join(dir, "icon.png");
fs.writeFileSync(pngPath, pngData);
console.log(`generated ${pngPath} (${pngData.length} bytes)`);
