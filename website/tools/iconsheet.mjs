#!/usr/bin/env node
// Dev-only: renders the brand SVGs into one inspection sheet (true sizes + pixel-exact zooms),
// plus the .ico and touch-icon rasters exactly as shipped.
// Usage: node iconsheet.mjs --brand DIR --out FILE.png
import { chromium } from 'playwright-core';
import fs from 'node:fs';
import path from 'node:path';

const a = Object.fromEntries(process.argv.slice(2).map((x, i, arr) => (x.startsWith('--') ? [x.slice(2), arr[i + 1]] : null)).filter(Boolean));
const brand = a.brand || '/out/brand', outFile = a.out || '/out/brand-sheet.png';
const LIGHT = '#F5F2EA', DARK = '#111F1B';
const inkOn = (bg) => (bg === DARK ? '#B7C5BC' : '#506058');
const read = (n) => fs.readFileSync(path.join(brand, n), 'utf8');
const sized = (svg, w, h) => svg.replace(/<\?xml[^>]*>\s*/, '').replace('<svg ', `<svg width="${w}" height="${h}" `);
const svg64 = (s) => 'data:image/svg+xml;base64,' + Buffer.from(s).toString('base64');
const png64 = (b) => 'data:image/png;base64,' + Buffer.from(b).toString('base64');
const cell = (bg, inner, w, pad, label) => `<figure style="margin:0;background:${bg};padding:${pad}px;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:14px;min-width:${w}px;box-sizing:border-box"><div style="display:flex;align-items:center;justify-content:center">${inner}</div>${label ? `<figcaption style="font:12px/1.35 ui-monospace,monospace;color:${inkOn(bg)};text-align:center;max-width:700px">${label}</figcaption>` : ''}</figure>`;
const favicon = read('favicon.svg'), app = read('app-icon.svg');
const sizes = [16, 24, 32, 128];
const row = (bg, svg) => `<div style="display:flex;align-items:flex-end;gap:28px">${sizes.map((s) => `<div style="display:flex;flex-direction:column;align-items:center;gap:8px">${sized(svg, s, s)}<span style="font:11px ui-monospace,monospace;color:${inkOn(bg)}">${s}</span></div>`).join('')}</div>`;
const zoom = (dataUrl, px, k) => `<canvas data-zoom="${dataUrl}" data-px="${px}" data-k="${k}" width="${px * k}" height="${px * k}" style="image-rendering:pixelated;outline:1px solid #888"></canvas>`;
const stack = (bg, svg) => `<div style="display:flex;flex-direction:column;align-items:center;gap:18px">${row(bg, svg)}<div style="display:flex;align-items:flex-start;gap:16px">${zoom(svg64(svg), 16, 8)}${zoom(svg64(svg), 24, 5)}</div><div style="display:flex;align-items:flex-start;gap:16px">${zoom(svg64(svg), 32, 4)}</div></div>`;

// Parse the .ico container so the frames that actually ship are what gets inspected.
let icoRow = '', icoZoom = '';
try {
  const ico = fs.readFileSync(path.join(brand, 'favicon.ico'));
  const n = ico.readUInt16LE(4);
  const frames = [];
  for (let i = 0; i < n; i++) { const o = 6 + i * 16; const size = ico.readUInt8(o) || 256; const len = ico.readUInt32LE(o + 8), off = ico.readUInt32LE(o + 12); const png = ico.subarray(off, off + len); frames.push({ size, isPng: png.readUInt32BE(0) === 0x89504e47, url: png64(png) }); }
  console.log('favicon.ico:', n, 'frames:', frames.map((f) => `${f.size}px png=${f.isPng}`).join(', '));
  icoRow = `<div style="display:flex;align-items:flex-end;gap:28px">${frames.map((f) => `<div style="display:flex;flex-direction:column;align-items:center;gap:8px"><img src="${f.url}" width="${f.size}" height="${f.size}" style="image-rendering:pixelated"><span style="font:11px ui-monospace,monospace;color:${inkOn(LIGHT)}">ico ${f.size}</span></div>`).join('')}</div>`;
  icoZoom = `<div style="display:flex;align-items:flex-start;gap:16px">${zoom(frames[0].url, frames[0].size, 8)}${zoom(frames[1].url, frames[1].size, 4)}</div>`;
} catch (e) { console.log('favicon.ico not parsed:', String(e.message)); }
const touch = fs.existsSync(path.join(brand, 'apple-touch-icon.png')) ? png64(fs.readFileSync(path.join(brand, 'apple-touch-icon.png'))) : null;

const html = `<!doctype html><meta charset="utf-8"><body style="margin:0;background:#888;font-family:ui-monospace,monospace">
<div style="display:flex;flex-wrap:wrap;gap:2px;width:1604px;padding:2px">
${cell(LIGHT, sized(read('join-icon.svg'), 220, 220), 396, 24, 'join-icon.svg (monochrome) on ivory')}
${cell(DARK, sized(read('join-icon-reversed.svg'), 220, 220), 396, 24, 'join-icon-reversed.svg on ink')}
${cell(LIGHT, sized(read('join-icon-two-tone.svg'), 220, 220), 396, 24, 'join-icon-two-tone.svg on ivory')}
${cell(DARK, sized(read('join-icon-two-tone-reversed.svg'), 220, 220), 396, 24, 'two-tone reversed on ink')}
${cell(LIGHT, sized(read('lockup.svg'), 288, 100), 396, 24, 'lockup.svg (monochrome)')}
${cell(DARK, sized(read('lockup-reversed.svg'), 288, 100), 396, 24, 'lockup-reversed.svg on ink')}
${cell(LIGHT, sized(read('lockup-two-tone.svg'), 288, 100), 396, 24, 'lockup-two-tone.svg (used in the site header)')}
${cell(DARK, sized(read('lockup-two-tone-reversed.svg'), 288, 100), 396, 24, 'lockup-two-tone-reversed.svg on ink')}
${cell(LIGHT, stack(LIGHT, favicon), 798, 20, 'favicon.svg at true 16 / 24 / 32 / 128 px on ivory; below: 16px x8, 24px x5, 32px x4 (nearest-neighbour zoom of the actual raster)')}
${cell(DARK, stack(DARK, favicon), 798, 20, 'favicon.svg at true 16 / 24 / 32 / 128 px on ink (dark tab strip)')}
${cell(LIGHT, `<div style="display:flex;flex-direction:column;align-items:center;gap:18px">${icoRow}${icoZoom}</div>`, 798, 20, 'favicon.ico as shipped: frames 16 / 32 / 48 on ivory; below: 16px frame x8 and 32px frame x4')}
${cell(DARK, `<div style="display:flex;flex-direction:column;align-items:center;gap:18px">${icoRow}${icoZoom}</div>`, 798, 20, 'favicon.ico as shipped on ink (dark tab strip)')}
${cell(LIGHT, sized(app, 192, 192) + sized(app, 96, 96) + sized(app, 64, 64) + sized(app, 48, 48), 1062, 24, 'app-icon.svg at 192 / 96 / 64 / 48 px on ivory')}
${cell(DARK, sized(app, 192, 192) + sized(app, 96, 96) + sized(app, 64, 64) + sized(app, 48, 48), 534, 24, 'app-icon.svg on ink')}
${touch ? cell(LIGHT, `<img src="${touch}" width="180" height="180"><img src="${touch}" width="120" height="120">`, 534, 24, 'apple-touch-icon.png as shipped (180 px, full-bleed square; iOS applies its own mask)') : ''}
${cell(LIGHT, sized(read('lockup-two-tone.svg'), 176, 61), 534, 24, 'header size 176x61 on ivory')}
${cell(DARK, sized(read('lockup-two-tone-reversed.svg'), 176, 61), 534, 24, 'header size on ink')}
${cell(LIGHT, sized(read('lockup-two-tone.svg'), 132, 46), 530, 24, 'small header 132x46 (mobile) on ivory')}
${cell(DARK, sized(read('lockup-two-tone-reversed.svg'), 132, 46), 530, 24, 'small header on ink')}
</div></body>`;
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1608, height: 1400 }, deviceScaleFactor: 1 });
await page.setContent(html);
await page.evaluate(async () => { for (const c of document.querySelectorAll('canvas[data-zoom]')) { const img = new Image(); await new Promise((r) => { img.onload = r; img.src = c.dataset.zoom; }); const px = Number(c.dataset.px), k = Number(c.dataset.k); const small = document.createElement('canvas'); small.width = px; small.height = px; small.getContext('2d').drawImage(img, 0, 0, px, px); const ctx = c.getContext('2d'); ctx.imageSmoothingEnabled = false; ctx.drawImage(small, 0, 0, px * k, px * k); } });
await page.waitForTimeout(300);
await page.screenshot({ path: outFile, fullPage: true });
await browser.close();
console.log('wrote', outFile);
