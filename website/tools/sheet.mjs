#!/usr/bin/env node
// Dev-only: lay a tall screenshot out in columns (a contact sheet) for whole-page review.
// Usage: node sheet.mjs IN.png COLUMN_HEIGHT GAP OUT.png   (paths relative to /out)
import { chromium } from 'playwright-core';
import fs from 'node:fs';
const [src, colH, gap, out] = process.argv.slice(2);
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 2600, height: 2600 }, deviceScaleFactor: 1 });
await page.setContent('<style>html,body{margin:0;background:#777}#s{display:flex;align-items:flex-start}</style><div id="s"></div>');
const info = await page.evaluate(async ({ colH, gap, data }) => {
  const img = new Image(); img.src = 'data:image/png;base64,' + data;
  await new Promise((r, j) => { img.onload = r; img.onerror = j; });
  const n = Math.ceil(img.naturalHeight / colH);
  const s = document.getElementById('s'); s.style.gap = gap + 'px'; s.style.padding = gap + 'px';
  for (let i = 0; i < n; i++) { const d = document.createElement('div'); d.style.cssText = `flex:none;width:${img.naturalWidth}px;height:${Math.min(colH, img.naturalHeight - i * colH)}px;background:url(${JSON.stringify('data:image/png;base64,' + data)}) 0 -${i * colH}px no-repeat`; s.appendChild(d); }
  return { columns: n, w: img.naturalWidth, h: img.naturalHeight };
}, { colH: Number(colH), gap: Number(gap), data: fs.readFileSync('/out/' + src).toString('base64') });
await page.waitForTimeout(300);
const size = await page.evaluate(() => ({ w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight }));
await page.setViewportSize({ width: Math.min(size.w, 16000), height: Math.min(size.h, 16000) });
await page.screenshot({ path: '/out/' + out, fullPage: true });
await browser.close();
console.log('wrote', out, JSON.stringify(info));
