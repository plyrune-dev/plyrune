#!/usr/bin/env node
// Dev-only: rasterises brand/app-icon.svg and favicon.svg with headless Chromium and writes
// favicon.ico (16, 32, 48 px, PNG frames) and apple-touch-icon.png (180 px, full-bleed square).
// Usage: node render-icons.mjs --brand DIR   (reads and writes SVG/PNG/ICO in DIR)
import { chromium } from 'playwright-core';
import fs from 'node:fs';
import path from 'node:path';

const a = Object.fromEntries(process.argv.slice(2).map((x, i, arr) => (x.startsWith('--') ? [x.slice(2), arr[i + 1]] : null)).filter(Boolean));
const dir = a.brand || '/out/brand';
const read = (n) => fs.readFileSync(path.join(dir, n), 'utf8');

const browser = await chromium.launch();
async function raster(svg, size) {
  const ctx = await browser.newContext({ viewport: { width: size, height: size }, deviceScaleFactor: 1 });
  const page = await ctx.newPage();
  const body = svg.replace(/<\?xml[^>]*>\s*/, '').replace('<svg ', `<svg width="${size}" height="${size}" `);
  await page.setContent(`<!doctype html><style>html,body{margin:0;background:transparent}svg{display:block}</style>${body}`);
  const buf = await page.screenshot({ type: 'png', omitBackground: true });
  await ctx.close();
  return buf;
}

// .ico container with PNG frames: 6-byte header, 16-byte directory entries, then the images.
function ico(frames) {
  const header = Buffer.alloc(6); header.writeUInt16LE(0, 0); header.writeUInt16LE(1, 2); header.writeUInt16LE(frames.length, 4);
  const dir = Buffer.alloc(16 * frames.length);
  let offset = 6 + dir.length;
  frames.forEach(({ size, png }, i) => {
    dir.writeUInt8(size >= 256 ? 0 : size, i * 16); dir.writeUInt8(size >= 256 ? 0 : size, i * 16 + 1);
    dir.writeUInt16LE(1, i * 16 + 4); dir.writeUInt16LE(32, i * 16 + 6); dir.writeUInt32LE(png.length, i * 16 + 8); dir.writeUInt32LE(offset, i * 16 + 12);
    offset += png.length;
  });
  return Buffer.concat([header, dir, ...frames.map((f) => f.png)]);
}

const out = {};
for (const s of [16, 32, 48]) out[`favicon-${s}.png`] = await raster(read('favicon.svg'), s);
const icoBuf = ico([16, 32, 48].map((s) => ({ size: s, png: out[`favicon-${s}.png`] })));
fs.writeFileSync(path.join(dir, 'favicon.ico'), icoBuf);
// The touch icon is full-bleed: iOS masks icons itself and shows black behind transparent corners.
const touch = read('app-icon.svg').replace('rx="22"', 'rx="0"').replace('translate(14 18.2) scale(0.96)', 'translate(23 26.03) scale(0.72)');
fs.writeFileSync(path.join(dir, 'apple-touch-icon.png'), await raster(touch, 180));
await browser.close();
console.log('favicon.ico', icoBuf.length, 'bytes; apple-touch-icon.png', fs.statSync(path.join(dir, 'apple-touch-icon.png')).size, 'bytes');
