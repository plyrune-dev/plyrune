#!/usr/bin/env node
// Dev-only: crop and enlarge a region of a PNG under /out (for pixel-level inspection).
// Usage: node crop.mjs IN.png X Y W H SCALE OUT.png   (paths relative to /out)
import { chromium } from 'playwright-core';
const [src, x, y, w, h, scale, out] = process.argv.slice(2);
const browser = await chromium.launch();
const ctx = await browser.newContext({ deviceScaleFactor: Number(scale), viewport: { width: 2600, height: 2600 } });
const page = await ctx.newPage();
await page.goto('file:///out/' + src);
await page.evaluate(() => { document.body.style.margin = '0'; const img = document.querySelector('img'); img.style.display = 'block'; img.style.imageRendering = 'pixelated'; });
const size = await page.evaluate(() => { const img = document.querySelector('img'); return { width: img.naturalWidth, height: img.naturalHeight }; });
await page.setViewportSize({ width: Math.min(size.width, 16000), height: Math.min(Math.max(size.height, 2600), 16000) });
await page.screenshot({ path: '/out/' + out, clip: { x: Number(x), y: Number(y), width: Number(w), height: Number(h) } });
await browser.close();
console.log('wrote', out, 'from', JSON.stringify(size));
