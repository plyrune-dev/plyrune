#!/usr/bin/env node
// Development-only checker and screenshot runner for the Plyrune website. Never deployed.
//
// It loads the site in headless browsers and reports REAL measurements. A clean run is not a WCAG
// conformance claim: items that need a human (meaning of alt text, status conveyed by colour alone,
// reading order, real assistive technology, real devices) are listed as manual in DESIGN.md.
//
// Usage (inside the pinned Playwright image; see tools/README.md):
//   node sitecheck.mjs [--root DIR] [--pages a.html,b.html] [--modes file,root,subpath] [--url URL]
//     [--out DIR] [--prefix NAME] [--toggle SELECTOR] [--shots] [--shotMode file|live]
//     [--browsers chromium,firefox,webkit] [--font segoe|inter|wide|container] [--full]
//     [--pairs ink/paper,muted/paper,...]   (design-token contrast matrix, both themes)
import { chromium, firefox, webkit } from 'playwright-core';
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';

const args = process.argv.slice(2);
const opt = { root: '/site', pages: 'index.html', modes: 'file', out: '/out/check', prefix: 'run', browsers: 'chromium', font: 'segoe', pairs: '' };
for (let i = 0; i < args.length; i++) {
  const k = args[i].replace(/^--/, '');
  const v = i + 1 < args.length && !args[i + 1].startsWith('--') ? args[++i] : true;
  opt[k] = v;
}
const AXE = process.env.AXE || '/tooling/node_modules/axe-core/axe.min.js';
const SHOTS = 'shots' in opt, RUN_AXE = !('no-axe' in opt), FULL = 'full' in opt;
const NAMED = [
  { name: 'desktop', width: 1440, height: 900, mobile: false },
  { name: 'tablet', width: 820, height: 1180, mobile: false },
  { name: 'mobile', width: 390, height: 844, mobile: true },
];
// Layout-only sizes: small phones, 200 % / 400 % zoom equivalents of a 1280 px window, large desktop.
const EXTRA = FULL ? [
  { name: 'w320', width: 320, height: 640, mobile: true }, { name: 'w360', width: 360, height: 740, mobile: true },
  { name: 'w640', width: 640, height: 360, mobile: false }, { name: 'w768', width: 768, height: 1024, mobile: false },
  { name: 'w1024', width: 1024, height: 768, mobile: false }, { name: 'w1920', width: 1920, height: 1080, mobile: false },
] : [{ name: 'w320', width: 320, height: 640, mobile: true }];
const THEMES = ['light', 'dark'];
const WCAG_TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
let servers = {}, modes = [], wording = [], weights = { files: {} }, summaryWritten = false;
const report = { runs: [], summary: {} };

// ---- Fonts. The site uses a system font stack. The container's default (WenQuanYi Zen Hei) is not
// representative, so tests run with documented stand-in fonts. Nothing here ships with the site.
const FONT_SETS = {
  segoe: { ui: 'Selawik', mono: 'JetBrains Mono', note: 'Selawik 1.01 (Segoe UI metric-compatible, MIT; 600 -> Selawik Semibold) + JetBrains Mono 2.304 (OFL)' },
  inter: { ui: 'Inter', mono: 'JetBrains Mono', note: 'Inter 4.0 (SF-like, OFL) + JetBrains Mono 2.304 (OFL)' },
  wide: { ui: 'DejaVu Sans', mono: 'JetBrains Mono', note: 'DejaVu Sans 2.37 (wide stress font) + JetBrains Mono 2.304 (OFL)' },
  container: null,
};
function writeFontconfig(profile) {
  const set = FONT_SETS[profile] || FONT_SETS.segoe;
  if (profile === 'container') return null;
  const rule = (from, to) => `  <match target="pattern"><test name="family" qual="any"><string>${from}</string></test><edit name="family" mode="prepend" binding="strong"><string>${to}</string></edit></match>`;
  const semibold = profile === 'segoe' ? `  <match target="pattern"><test name="family" qual="any"><string>Selawik</string></test><test name="weight" compare="more_eq"><const>demibold</const></test><test name="weight" compare="less"><const>bold</const></test><edit name="family" mode="assign" binding="strong"><string>Selawik Semibold</string></edit></match>\n` : '';
  const conf = `<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <dir>/usr/share/fonts</dir>
  <dir>/tooling/fonts/fonts</dir>
  <dir>/tooling/selawik</dir>
  <cachedir>/tmp/fontconfig-cache</cachedir>
  <include ignore_missing="yes">/etc/fonts/conf.d</include>
${['system-ui', 'ui-sans-serif', 'Segoe UI', '-apple-system', 'BlinkMacSystemFont', 'Roboto', 'Helvetica Neue', 'Noto Sans'].map((f) => rule(f, set.ui)).join('\n')}
${['ui-monospace', 'monospace', 'SF Mono', 'SFMono-Regular', 'Cascadia Code', 'Segoe UI Mono', 'Roboto Mono', 'Menlo', 'Consolas', 'DejaVu Sans Mono'].map((f) => rule(f, set.mono)).join('\n')}
${semibold}}</fontconfig>
`;
  const file = `/tmp/fonts-${profile}.conf`;
  fs.writeFileSync(file, conf);
  process.env.FONTCONFIG_FILE = file;
  return conf;
}
const FONTCONF = writeFontconfig(opt.font);

// ---- local static server that mimics GitHub Pages for root and sub-path hosting
const MIME = { '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8', '.svg': 'image/svg+xml', '.png': 'image/png', '.ico': 'image/x-icon', '.json': 'application/json; charset=utf-8', '.webmanifest': 'application/manifest+json', '.txt': 'text/plain; charset=utf-8', '.woff2': 'font/woff2', '.xml': 'application/xml' };
function startServer(root, prefix) {
  const log = [];
  const srv = http.createServer((req, res) => {
    const u = new URL(req.url, 'http://localhost');
    let p;
    try { p = decodeURIComponent(u.pathname); } catch { res.writeHead(400); res.end(); return; }
    const entry = { method: req.method, path: u.pathname, status: 0, bytes: 0 };
    log.push(entry);
    const send = (status, data, type) => { entry.status = status; entry.bytes = data.length; entry.type = type; res.writeHead(status, { 'Content-Type': type, 'Content-Length': data.length, 'Cache-Control': 'max-age=600' }); res.end(req.method === 'HEAD' ? undefined : data); };
    if (prefix) {
      if (p === prefix.slice(0, -1)) { entry.status = 301; entry.location = prefix + u.search; res.writeHead(301, { Location: prefix + u.search }); res.end(); return; }
      if (!p.startsWith(prefix)) { send(404, Buffer.from('outside site prefix'), 'text/plain'); return; }
      p = '/' + p.slice(prefix.length);
    }
    let fp = path.normalize(path.join(root, p));
    if (!fp.startsWith(root)) { send(403, Buffer.from('forbidden'), 'text/plain'); return; }
    try { if (fs.statSync(fp).isDirectory()) fp = path.join(fp, 'index.html'); } catch { /* not a directory */ }
    fs.readFile(fp, (err, data) => {
      if (err) { const nf = path.join(root, '404.html'); if (fs.existsSync(nf)) fs.readFile(nf, (e2, d2) => e2 ? send(404, Buffer.from('not found'), 'text/plain') : send(404, d2, MIME['.html'])); else send(404, Buffer.from('not found'), 'text/plain'); return; }
      send(200, data, MIME[path.extname(fp).toLowerCase()] || 'application/octet-stream');
    });
  });
  return new Promise((resolve) => srv.listen(0, '127.0.0.1', () => resolve({ srv, log, port: srv.address().port, close: () => new Promise((r) => srv.close(r)) })));
}

// ---- in-page probes (serialised into the page; must be self-contained)
const LAYOUT_PROBE = () => {
  const de = document.documentElement;
  const vw = de.clientWidth;
  const out = { vw, innerW: window.innerWidth, innerH: window.innerHeight, scrollW: de.scrollWidth, pageH: de.scrollHeight, overflowers: [], clipped: [], textOverlaps: [], targets: [], zeroSizeTargets: [], headings: [], landmarks: [] };
  const sig = (el) => { let s = el.tagName.toLowerCase(); if (el.id) s += '#' + el.id; if (typeof el.className === 'string' && el.className.trim()) s += '.' + el.className.trim().split(/\s+/).slice(0, 3).join('.'); const t = (el.getAttribute('aria-label') || el.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 36); return s + (t ? ` "${t}"` : ''); };
  const inScroller = (el) => { let p = el.parentElement; while (p && p !== document.body) { const s = getComputedStyle(p); if (/(auto|scroll|hidden)/.test(s.overflowX) && p.scrollWidth > p.clientWidth + 1) return true; p = p.parentElement; } return false; };
  const rects = [];
  // Chromium keeps layout boxes for content hidden by a closed <details>; it is not rendered.
  const concealed = (el) => !!(el.closest && el.closest('details:not([open])') && !el.closest('summary'));
  for (const el of document.querySelectorAll('body *')) {
    if (el.namespaceURI !== 'http://www.w3.org/1999/xhtml' || concealed(el)) continue;
    const cs = getComputedStyle(el);
    if (cs.display === 'none' || cs.visibility === 'hidden') continue;
    const r = el.getBoundingClientRect();
    if (r.width === 0 && r.height === 0) continue;
    const tiny = r.width <= 2 && r.height <= 2; // visually-hidden helper text is intentional
    if (!tiny && (r.right > vw + 1 || r.left < -1) && !inScroller(el)) out.overflowers.push(`${sig(el)} [${Math.round(r.left)}..${Math.round(r.right)}]`);
    if (!tiny && cs.display !== 'inline' && cs.position !== 'absolute') {
      const cutX = /(hidden|clip)/.test(cs.overflowX) && el.scrollWidth > el.clientWidth + 1;
      const cutY = /(hidden|clip)/.test(cs.overflowY) && el.scrollHeight > el.clientHeight + 1;
      if (cutX || cutY || (cs.textOverflow === 'ellipsis' && el.scrollWidth > el.clientWidth + 1)) out.clipped.push(`${sig(el)} scroll=${el.scrollWidth}x${el.scrollHeight} client=${el.clientWidth}x${el.clientHeight}`);
    }
  }
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  let node;
  while ((node = walker.nextNode())) {
    const val = node.nodeValue;
    if (!val || !val.trim()) continue;
    const p = node.parentElement;
    if (!p || concealed(p) || /^(script|style|noscript|template|title)$/i.test(p.tagName)) continue;
    const cs = getComputedStyle(p);
    if (cs.display === 'none' || cs.visibility === 'hidden') continue;
    const range = document.createRange();
    range.selectNodeContents(node);
    for (const r of range.getClientRects()) {
      if (r.width > 2 && r.height > 2) rects.push({ l: r.left + 1, t: r.top + 1, r: r.right - 1, b: r.bottom - 1, w: r.width, h: r.height, s: `${p.tagName.toLowerCase()}${p.className && typeof p.className === 'string' ? '.' + p.className.trim().split(/\s+/)[0] : ''} "${val.trim().replace(/\s+/g, ' ').slice(0, 24)}"` });
    }
  }
  for (let i = 0; i < rects.length; i++) for (let j = i + 1; j < rects.length; j++) {
    const a = rects[i], b = rects[j];
    const w = Math.min(a.r, b.r) - Math.max(a.l, b.l), h = Math.min(a.b, b.b) - Math.max(a.t, b.t);
    // ignore the slight bounding-box overlap of tightly leaded display lines (ink does not touch)
    if (w > 2 && h > 2 && h > 0.22 * Math.min(a.h, b.h) && w > 0.22 * Math.min(a.w, b.w)) { out.textOverlaps.push(`${a.s} <> ${b.s} (${Math.round(w)}x${Math.round(h)})`); if (out.textOverlaps.length > 40) return out; }
  }
  for (const el of document.querySelectorAll('a[href],button,summary,input,select,textarea,[tabindex]:not([tabindex="-1"])')) {
    if (concealed(el)) continue;
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    if (cs.display === 'none' || cs.visibility === 'hidden') continue;
    if (r.width === 0 || r.height === 0) { out.zeroSizeTargets.push(sig(el)); continue; }
    out.targets.push({ sig: sig(el), w: Math.round(r.width * 10) / 10, h: Math.round(r.height * 10) / 10, inline: cs.display === 'inline', offscreen: r.bottom < 0 || r.top > window.innerHeight || r.right < 0 || r.left > window.innerWidth });
  }
  for (const h of document.querySelectorAll('h1,h2,h3,h4,h5,h6')) out.headings.push(`${h.tagName.toLowerCase()}: ${(h.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 70)}`);
  for (const l of document.querySelectorAll('header,nav,main,footer,aside,[role=banner],[role=navigation],[role=main],[role=complementary],[role=contentinfo],[role=search],[role=region],[role=group][aria-label]')) out.landmarks.push(`${l.tagName.toLowerCase()}${l.getAttribute('aria-label') ? `[aria-label=${l.getAttribute('aria-label')}]` : l.id ? `#${l.id}` : ''}`);
  const lum = (c) => { const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }; return 0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2]); };
  const bg = getComputedStyle(document.body).backgroundColor.match(/[\d.]+/g).map(Number);
  out.bodyBg = getComputedStyle(document.body).backgroundColor;
  out.bodyBgLuminance = bg.length >= 3 ? Math.round(lum(bg) * 1000) / 1000 : null;
  out.effectiveTheme = out.bodyBgLuminance === null ? 'unknown' : out.bodyBgLuminance > 0.4 ? 'light' : out.bodyBgLuminance < 0.1 ? 'dark' : 'mid';
  out.rootAttr = [...document.documentElement.attributes].filter((a) => /theme|color|scheme|data-/.test(a.name)).map((a) => `${a.name}=${a.value}`).join(' ');
  return out;
};

const REF_PROBE = () => {
  const out = [];
  const add = (kind, raw) => { if (raw != null && String(raw).trim() !== '') out.push([kind, String(raw).trim()]); };
  for (const a of document.querySelectorAll('[href]')) add(a.tagName.toLowerCase() + '[href]', a.getAttribute('href'));
  for (const a of document.querySelectorAll('[src]')) add(a.tagName.toLowerCase() + '[src]', a.getAttribute('src'));
  for (const a of document.querySelectorAll('[srcset]')) add('srcset', a.getAttribute('srcset'));
  for (const a of document.querySelectorAll('[poster],[ping],[srcdoc],[data],[action],[manifest]')) for (const at of ['poster', 'ping', 'srcdoc', 'data', 'action', 'manifest']) add(a.tagName.toLowerCase() + '[' + at + ']', a.getAttribute(at));
  for (const a of document.querySelectorAll('[style]')) add('style-attr', a.getAttribute('style'));
  for (const s of document.querySelectorAll('style')) { for (const m of (s.textContent || '').matchAll(/url\(\s*['"]?([^'")]+)['"]?\s*\)/g)) add('inline-css-url', m[1]); for (const m of (s.textContent || '').matchAll(/@import\s+['"]([^'"]+)['"]/g)) add('css-import', m[1]); }
  const html = document.documentElement.outerHTML;
  // Real URL references live in attribute values (never xmlns) and in inline script/style text.
  const URL_RE = /(?:https?:)?\/\/[^\s"'<>)\\]+/g;
  for (const el of document.querySelectorAll('*')) for (const at of el.attributes) { if (/^xmlns(:|$)/i.test(at.name)) continue; for (const m of at.value.matchAll(URL_RE)) add('absolute-url-in-attribute', m[0]); }
  for (const s of document.querySelectorAll('script,style')) for (const m of (s.textContent || '').matchAll(URL_RE)) add('absolute-url-in-inline-' + s.tagName.toLowerCase(), m[0]);
  return { refs: out, meta: { inlineScripts: document.querySelectorAll('script:not([src])').length, externalScripts: [...document.querySelectorAll('script[src]')].map((s) => s.getAttribute('src')), inlineStyles: document.querySelectorAll('style').length, inlineEventHandlers: [...document.querySelectorAll('*')].filter((el) => [...el.attributes].some((a) => /^on/i.test(a.name))).length, metaRefresh: !!document.querySelector('meta[http-equiv="refresh" i]'), embeds: document.querySelectorAll('iframe,frame,object,embed').length, imgsWithoutAlt: document.querySelectorAll('img:not([alt])').length, images: document.querySelectorAll('img').length, svgsWithoutName: [...document.querySelectorAll('svg')].filter((s) => s.closest('[aria-hidden="true"]') || s.closest('button,a,[role=img]') ? false : !(s.getAttribute('aria-label') || s.getAttribute('aria-labelledby') || s.querySelector('title'))).length, docTitle: document.title, lang: document.documentElement.lang, metaDescription: (document.querySelector('meta[name="description" i]') || {}).content || null, viewportMeta: (document.querySelector('meta[name="viewport" i]') || {}).content || null, faviconLinks: [...document.querySelectorAll('link[rel*="icon" i]')].map((l) => l.getAttribute('href')), ids: [...document.querySelectorAll('[id]')].map((e) => e.id), text: (document.body.innerText || document.body.textContent || '') } };
};

const TOKEN_PROBE = (pairs) => {
  const norm = (prop) => { const el = document.createElement('span'); el.style.color = `var(${prop})`; el.style.backgroundColor = 'transparent'; document.body.appendChild(el); const v = getComputedStyle(el).color; el.remove(); return v; };
  const lum = (c) => { const m = c.match(/[\d.]+/g).map(Number); if (m.length < 3 || m[3] === 0) return null; const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }; return 0.2126 * f(m[0]) + 0.7152 * f(m[1]) + 0.0722 * f(m[2]); };
  const defined = (prop) => { for (const el of [document.documentElement, document.body]) { if (getComputedStyle(el).getPropertyValue(prop).trim()) return getComputedStyle(el).getPropertyValue(prop).trim(); } return ''; };
  return pairs.map(([fg, bg]) => { const fgRaw = defined(`--${fg}`), bgRaw = defined(`--${bg}`); if (!fgRaw || !bgRaw) return { fg, bg, fgRaw, bgRaw, ratio: null, note: 'token not defined on this page' }; const a = norm(`--${fg}`), b = norm(`--${bg}`); const la = lum(a), lb = lum(b); const [x, y] = [la, lb].sort((p, q) => q - p); return { fg, bg, fgRaw, bgRaw, fgComputed: a, bgComputed: b, ratio: la === null || lb === null ? null : Math.round(((x + 0.05) / (y + 0.05)) * 100) / 100 }; });
};

const PREVIEW_INSTALL = 'curl -fsSL https://plyrune-dev.github.io/install.sh | sh';
const RELEASE_COMMANDS = [PREVIEW_INSTALL, '"$HOME/.local/bin/plyrune" demo'];
const FORBIDDEN = [['verified agent', /verified agent/i], ['correct edits', /correct edits/i], ['safe at any time', /safe at any time/i], ['all callers', /all callers/i], ['prevents stale edits', /prevents stale edits/i], ['unrelated GitHub URL', /github\.com\/(?!plyrune-dev\/plyrune(?:[/?#\s]|$))/i], ['install command', /\b(npm|cargo|brew|pip|apt|pnpm|yarn)\s+install\b|\bcurl\b[^\n]*\|\s*(ba)?sh\b/i], ['percent speedup', /\d+(\.\d+)?\s?%\s+(faster|slower|speedup|improvement|cheaper|fewer|less|more\b)/i], ['Nx faster', /\d+(\.\d+)?\s?[x×]\s+(faster|speedup|cheaper)/i], ['customers', /trusted by|customer logos?|\b\d+\s+customers?\b/i], ['stars', /\b\d[\d,.]*\s?(k\s)?stars?\b/i], ['trial', /\bfree trial\b|\b\d+\s+day trial\b|\btrial\s+count\b/i]];
const WATCH = [/proof/i, /\bprove[ds]?\b|\bproven\b|proofs/i, /verif/i, /guarantee/i, /\bsafe\b|\bsafely\b/i, /\bcorrect(ly|ness)?\b/i, /\binstall/i, /\bfast(er)?\b|speed|performance/i, /\d+\s?%/, /\bfaster\b|\bcheaper\b|\breduce[sd]?\b|\bsave[sd]?\b/i, /\bgithub\b|\brepo(sitory)?\b/i];

function wordingLint(text, label, required) {
  const flat = text.replace(/\s+/g, ' ');
  const res = { label, missing: required.filter((s) => !flat.includes(s)), forbidden: [], watch: {} };
  // Only this one published, checksum-pinned preview installer is qualified.
  const checked = flat.replaceAll(PREVIEW_INSTALL, '');
  for (const [name, re] of FORBIDDEN) { const m = checked.match(re); if (m) res.forbidden.push(`${name}: "${m[0]}"`); }
  for (const re of WATCH) { const ms = [...flat.matchAll(new RegExp(re.source, re.flags.includes('g') ? re.flags : re.flags + 'g'))]; if (ms.length) res.watch[re.source] = ms.map((m) => { const i = m.index; return '…' + flat.slice(Math.max(0, i - 38), i + m[0].length + 38).trim() + '…'; }); }
  return res;
}

function classifyRefs(refs, { root, pageDir }) {
  const rows = [];
  const seen = new Set();
  for (const [kind, raw] of refs) {
    if (kind === 'style-attr') { rows.push({ kind, ref: raw.slice(0, 60), verdict: 'inline style attribute' }); continue; }
    for (const part of kind === 'srcset' ? raw.split(',') : [raw]) {
      const ref = part.trim().split(/\s+/)[0];
      if (!ref || seen.has(kind + ref)) continue;
      seen.add(kind + ref);
      let verdict = 'relative';
      if (ref.startsWith('#')) verdict = 'fragment';
      else if (/^(https?:)?\/\//i.test(ref)) verdict = 'absolute';
      else if (/^[a-z][a-z0-9+.-]*:/i.test(ref)) verdict = 'scheme:' + ref.split(':')[0];
      else if (ref.startsWith('/')) verdict = 'root-absolute';
      let exists = null;
      if (verdict === 'relative' && pageDir) { const fp = path.resolve(pageDir, ref.split('#')[0].split('?')[0]); exists = fp.startsWith(root) && fs.existsSync(fp) ? fs.statSync(fp).size : false; if (exists === false) verdict = 'MISSING-or-outside-root'; }
      rows.push({ kind, ref: ref.slice(0, 100), verdict, bytes: typeof exists === 'number' ? exists : undefined });
    }
  }
  return rows;
}

async function runAxe(page, runOnly) {
  return page.evaluate(async (runOnly) => {
    const r = await axe.run(document, { runOnly, resultTypes: ['violations', 'incomplete', 'passes'] });
    const slim = (list) => list.map((v) => ({ id: v.id, impact: v.impact, help: v.help, tags: v.tags.filter((t) => /wcag\d|best-practice/.test(t)), nodes: v.nodes.map((n) => ({ target: Array.isArray(n.target) ? n.target.join(' ') : String(n.target), html: (n.html || '').slice(0, 140), summary: (n.failureSummary || '').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').slice(0, 220), any: n.any.map((c) => c.message.slice(0, 160)) })) }));
    return { violations: slim(r.violations), incomplete: slim(r.incomplete), passes: r.passes.map((p) => ({ id: p.id, tags: p.tags.filter((t) => /wcag\d|best-practice/.test(t)), n: p.nodes.length })) };
  }, runOnly);
}

// Wait until the window scroll position has stopped changing (focus/hover trigger smooth scrolling).
const settleScroll = (page) => page.evaluate(() => new Promise((resolve) => { let last = window.scrollY, stable = 0; const t0 = performance.now(); const tick = () => { const y = window.scrollY; if (y === last) stable += 1; else { stable = 0; last = y; } if (stable >= 6 || performance.now() - t0 > 2000) resolve(); else requestAnimationFrame(tick); }; requestAnimationFrame(tick); }));

async function focusPass(page) {
  const sel = 'a[href],button,summary,input,select,textarea,[tabindex]:not([tabindex="-1"])';
  const n = await page.evaluate((s) => document.querySelectorAll(s).length, sel);
  const seen = [];
  for (let i = 0; i < n + 1; i++) {
    await page.keyboard.press('Tab');
    await settleScroll(page);
    const info = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body || el === document.documentElement) return null;
      const cs = getComputedStyle(el);
      const r = el.getBoundingClientRect();
      const cx = Math.min(innerWidth - 1, Math.max(0, r.left + r.width / 2)), cy = Math.min(innerHeight - 1, Math.max(0, r.top + r.height / 2));
      const top = document.elementFromPoint(cx, cy);
      const name = el.getAttribute('aria-label') || (el.innerText || el.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 40);
      return { name: `${el.tagName.toLowerCase()}:${el.getAttribute('href') || ''} "${name}"`, outlineStyle: cs.outlineStyle, outlineWidth: cs.outlineWidth, outlineColor: cs.outlineColor, shadow: cs.boxShadow, w: Math.round(r.width), h: Math.round(r.height), obscured: !(top && (top === el || el.contains(top) || top.contains(el))), inView: r.top >= -1 && r.left >= -1 && r.bottom <= innerHeight + 1 && r.right <= innerWidth + 1, focusVisible: el.matches(':focus-visible') };
    });
    if (!info) break;
    if (seen.length && seen[0].name === info.name) break;
    let contrast = null;
    if (RUN_AXE) { const a = await runAxe(page, { type: 'rule', values: ['color-contrast'] }); contrast = a.violations.flatMap((v) => v.nodes.map((nd) => `${nd.target}: ${nd.any.join(' | ')}`)); }
    seen.push({ ...info, contrastViolations: contrast });
  }
  await page.evaluate(() => { if (document.activeElement && document.activeElement.blur) document.activeElement.blur(); window.scrollTo(0, 0); });
  return seen;
}

async function hoverPass(page) {
  const sel = 'a[href],button,summary';
  const n = await page.evaluate((s) => document.querySelectorAll(s).length, sel);
  const rows = [];
  for (let i = 0; i < n; i++) {
    const loc = page.locator(sel).nth(i);
    if (!(await loc.count())) continue;
    const label = await loc.evaluate((el) => { const r = el.getBoundingClientRect(); return { name: `${el.tagName.toLowerCase()}:${el.getAttribute('href') || ''} "${(el.innerText || el.textContent || '').trim().replace(/\s+/g, ' ').slice(0, 40)}"`, visible: r.width > 0 && r.height > 0, offscreen: r.bottom <= 0 || r.top >= innerHeight }; });
    if (!label.visible) { rows.push({ name: label.name, skipped: 'not visible (closed disclosure or hidden)' }); continue; }
    if (label.offscreen) { rows.push({ name: label.name, skipped: 'kept off-screen until focused (skip link)' }); continue; }
    try { await loc.scrollIntoViewIfNeeded(); await settleScroll(page); await loc.hover({ timeout: 3000 }); } catch (e) { rows.push({ name: label.name, error: 'hover failed: ' + String(e.message).split('\n')[0].slice(0, 100) }); continue; }
    const style = await loc.evaluate((el) => { const cs = getComputedStyle(el); return { color: cs.color, bg: cs.backgroundColor, deco: cs.textDecorationLine, thickness: cs.textDecorationThickness, outline: cs.outlineStyle }; });
    let contrast = null;
    if (RUN_AXE) { const a = await runAxe(page, { type: 'rule', values: ['color-contrast'] }); contrast = a.violations.flatMap((v) => v.nodes.map((nd) => `${nd.target}: ${nd.any.join(' | ')}`)); }
    rows.push({ name: label.name, ...style, contrastViolations: contrast });
  }
  await page.mouse.move(0, 0);
  await page.evaluate(() => window.scrollTo(0, 0));
  return rows;
}

const SPACING_CSS = '*{line-height:1.5!important;letter-spacing:.12em!important;word-spacing:.16em!important}p{margin-bottom:2em!important}';
async function textSpacing(page) {
  const tag = await page.addStyleTag({ content: SPACING_CSS });
  await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
  const r = await page.evaluate(LAYOUT_PROBE);
  await tag.evaluate((e) => e.remove());
  await page.evaluate(() => window.scrollTo(0, 0));
  return { scrollW: r.scrollW, vw: r.vw, overflowers: r.overflowers, clipped: r.clipped, textOverlaps: r.textOverlaps };
}

// Interactions that must work with JavaScript: theme toggle flips the effective theme both ways.
async function toggleTest(page) {
  const lum = () => page.evaluate(() => { const m = getComputedStyle(document.body).backgroundColor.match(/[\d.]+/g).map(Number); const f = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }; return Math.round((0.2126 * f(m[0]) + 0.7152 * f(m[1]) + 0.0722 * f(m[2])) * 1000) / 1000; });
  const btn = page.locator('button[aria-pressed], button[data-theme-toggle], #theme').first();
  if (!(await btn.count())) return { skipped: 'no theme toggle' };
  const before = await lum();
  const labelBefore = (await btn.innerText()).trim();
  await btn.click();
  const after1 = await lum();
  const labelAfter = (await btn.innerText()).trim();
  const attr = await page.evaluate(() => document.documentElement.getAttribute('data-theme'));
  const pressed = await btn.getAttribute('aria-pressed');
  await btn.click();
  const after2 = await lum();
  return { labelBefore, labelAfter, ariaPressed: pressed, dataTheme: attr, luminance: [before, after1, after2], flipsBothWays: Math.sign(before - after1) !== 0 && Math.abs(before - after2) < 0.01 };
}
async function disclosureTest(page) {
  const s = page.locator('summary').first();
  if (!(await s.count())) return { skipped: 'no disclosure' };
  const open = () => page.evaluate(() => document.querySelectorAll('details[open]').length);
  const summaryText = (await s.innerText()).trim();
  const o0 = await open();
  await s.click(); await settleScroll(page); const o1 = await open();
  await s.focus(); await page.keyboard.press('Enter'); const o2 = await open();
  await page.keyboard.press('Space'); const o3 = await open();
  return { summaryText, openCount: [o0, o1, o2, o3], togglesByClick: o1 !== o0, togglesByEnter: o2 !== o1, togglesBySpace: o3 !== o2 };
}

async function main() {
  fs.mkdirSync(opt.out, { recursive: true });
  if (FONTCONF) fs.writeFileSync(path.join(opt.out, `${opt.prefix}-fonts.conf`), FONTCONF);
  const root = path.resolve(opt.root);
  const pages = String(opt.pages).split(',');
  modes = opt.url ? ['live'] : String(opt.modes).split(',');
  const browsers = String(opt.browsers).split(',');
  const pairs = String(opt.pairs || '').split(',').filter(Boolean).map((p) => p.split('/'));
  Object.assign(report, { generatedAt: new Date().toISOString(), root, pages, modes, browsers, fontProfile: opt.font, fontNote: (FONT_SETS[opt.font] || {}).note || 'container defaults', node: process.version });
  wording = []; weights = { files: {} };

  const walk = (d) => fs.readdirSync(d, { withFileTypes: true }).flatMap((e) => e.isDirectory() ? walk(path.join(d, e.name)) : [path.join(d, e.name)]);
  const TEXT = /\.(html|css|js|mjs|svg|json|txt|webmanifest)$/i;
  for (const f of walk(root).sort()) { const rel = path.relative(root, f); const buf = fs.readFileSync(f); weights.files[rel] = { raw: buf.length, gzip: TEXT.test(f) ? zlib.gzipSync(buf, { level: 9 }).length : undefined, brotli: TEXT.test(f) ? zlib.brotliCompressSync(buf, { params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 5 } }).length : undefined }; }
  const code = Object.entries(weights.files).filter(([k]) => /\.(html|css|js)$/i.test(k));
  weights.htmlCssJsRaw = code.reduce((s, [, v]) => s + v.raw, 0);
  weights.htmlCssJsBrotli = code.reduce((s, [, v]) => s + (v.brotli || 0), 0);
  weights.allRaw = Object.values(weights.files).reduce((s, v) => s + v.raw, 0);
  weights.nojekyll = Object.keys(weights.files).includes('.nojekyll');
  weights.cname = Object.keys(weights.files).includes('CNAME');

  for (const pageName of pages) {
    const htmlPath = path.join(root, pageName);
    if (!fs.existsSync(htmlPath)) { report.summary[`missing ${pageName}`] = true; continue; }
    const html = fs.readFileSync(htmlPath, 'utf8');
    const attrText = [...html.matchAll(/="([^"]*)"/g)].map((m) => m[1]).join(' ');
    const text = (html.replace(/<(script|style)\b[^>]*>[\s\S]*?<\/\1>/gi, ' ').replace(/<[^>]+>/g, ' ') + ' ' + attrText).replace(/\s+/g, ' ');
    wording.push(wordingLint(text, pageName, pageName === 'index.html' ? RELEASE_COMMANDS : []));
    for (const m of html.matchAll(/<link[^>]+rel=["']?stylesheet["']?[^>]*>/gi)) { const h = (m[0].match(/href=["']([^"']+)["']/) || [])[1]; if (h && !/^(https?:)?\/\//.test(h)) { const fp = path.resolve(path.dirname(htmlPath), h.split('?')[0]); if (fs.existsSync(fp)) { const css = fs.readFileSync(fp, 'utf8'); for (const u of css.matchAll(/url\(\s*['"]?([^'")]+)['"]?\s*\)/g)) report.cssUrls = (report.cssUrls || []).concat([[pageName, 'css-url', u[1]]]); for (const u of css.matchAll(/@import\s+['"]([^'"]+)['"]/g)) report.cssUrls = (report.cssUrls || []).concat([[pageName, 'css-import', u[1]]]); } } }
  }

  servers = {};
  for (const m of modes) { if (m === 'root') servers.root = await startServer(root, ''); else if (m === 'subpath') servers.subpath = await startServer(root, '/plyrune/'); }
  const originOf = (mode) => mode === 'root' ? `http://127.0.0.1:${servers.root.port}` : mode === 'subpath' ? `http://127.0.0.1:${servers.subpath.port}` : mode === 'live' ? new URL(String(opt.url)).origin : 'file://' + root;
  const baseFor = (mode, pageName) => mode === 'file' ? 'file://' + path.join(root, pageName) : mode === 'root' ? `http://127.0.0.1:${servers.root.port}/${pageName}` : mode === 'subpath' ? `http://127.0.0.1:${servers.subpath.port}/plyrune/${pageName}` : String(opt.url).replace(/\/$/, '') + '/' + (pageName === 'index.html' ? '' : pageName);

  for (const bName of browsers) {
    const type = { chromium, firefox, webkit }[bName];
    let browser;
    try {
      // A browser that never finishes launching is reported as a failure instead of blocking the run.
      browser = await Promise.race([type.launch(), new Promise((_, rej) => setTimeout(() => rej(new Error('launch did not complete within 90 s')), 90000))]);
    } catch (e) { report.summary[`launch ${bName}`] = 'FAILED: ' + String(e.message).split('\n')[0].slice(0, 200); continue; }
    report[bName + 'Version'] = browser.version();
    for (const mode of modes) for (const pageName of pages) {
      const htmlPath = path.join(root, pageName);
      if (!fs.existsSync(htmlPath)) continue;
      for (const vp of [...NAMED, ...EXTRA]) for (const theme of THEMES) {
        const named = NAMED.includes(vp);
        if (mode !== 'file' && !named) continue;                               // extra sizes only in file mode
        if (mode !== 'file' && mode !== 'live' && theme !== 'light') continue; // local http: one theme; live: both
        if (bName !== 'chromium' && !(named && (vp === NAMED[0] || vp === NAMED[2]))) continue;
        const ctx = await browser.newContext({ viewport: { width: vp.width, height: vp.height }, colorScheme: theme, deviceScaleFactor: 1, reducedMotion: 'no-preference', isMobile: vp.mobile, hasTouch: vp.mobile });
        const page = await ctx.newPage();
        const rec = { browser: bName, mode, page: pageName, viewport: `${vp.name} ${vp.width}x${vp.height}`, theme, url: baseFor(mode, pageName), console: [], pageErrors: [], failedRequests: [], badResponses: [], requests: [] };
        const t0 = Date.now();
        console.error(`[${bName} ${mode} ${pageName} ${rec.viewport} ${theme}] start`);
        page.on('console', (m) => { if (m.type() === 'error' || m.type() === 'warning') rec.console.push(`${m.type()}: ${m.text().slice(0, 200)}`); });
        page.on('pageerror', (e) => rec.pageErrors.push(String(e.message).slice(0, 200)));
        page.on('requestfailed', (r) => rec.failedRequests.push(`${r.url()} ${r.failure()?.errorText}`));
        page.on('response', (r) => { if (r.status() >= 400) rec.badResponses.push(`${r.status()} ${r.url()}`); });
        page.on('request', (r) => rec.requests.push(r.url()));
        page.on('requestfinished', async (r) => { try { const b = await r.response(); if (b) { const h = await b.allHeaders(); rec.responses = rec.responses || []; rec.responses.push({ url: r.url().slice(0, 120), status: b.status(), type: r.resourceType(), contentLength: h['content-length'] ? Number(h['content-length']) : null, encoding: h['content-encoding'] || null, body: (await b.body()).length }); } } catch { /* ignore */ } });
        try {
          const resp = await page.goto(rec.url, { waitUntil: 'load', timeout: 45000 });
          rec.status = resp ? resp.status() : null;
          if (resp && mode === 'live') rec.docHeaders = Object.fromEntries(Object.entries(await resp.allHeaders()).filter(([k]) => /content-type|content-security|cache-control|content-encoding|content-length|^server$|strict-transport|x-content-type|referrer-policy|permissions-policy/.test(k)));
          await page.evaluate(() => document.fonts.ready);
          await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
          // Everything below "load" is measured before any test tooling is injected into the page.
          rec.load = { requests: [...rec.requests], console: [...rec.console], failedRequests: [...rec.failedRequests], badResponses: [...rec.badResponses], pageErrors: [...rec.pageErrors] };
          if (bName === 'chromium' && named) {
            const probe = await page.evaluate(REF_PROBE);
            rec.meta = probe.meta; rec.textLength = probe.meta.text.length; delete probe.meta.text; rec.ids = probe.meta.ids;
            rec.refs = classifyRefs(probe.refs, { root, pageDir: mode === 'file' ? path.dirname(htmlPath) : null });
            if (pairs.length && mode === 'file') rec.tokens = await page.evaluate(TOKEN_PROBE, pairs);
          }
          if (RUN_AXE) {
            await page.addScriptTag({ path: AXE }); // injected from a local file; never fetched by the page
            if (!report.axeCatalog) report.axeCatalog = await page.evaluate(() => axe.getRules().map((r) => ({ id: r.ruleId, enabled: r.enabled, tags: r.tags })));
          }
          if (typeof opt.toggle === 'string' && theme === 'dark') await page.locator(opt.toggle).first().click();
          rec.layout = await page.evaluate(LAYOUT_PROBE);
          if (rec.layout.effectiveTheme !== theme) rec.themeMismatch = `asked ${theme}, body background ${rec.layout.bodyBg} luminance ${rec.layout.bodyBgLuminance}`;
          rec.externalRequests = rec.load.requests.filter((u) => !u.startsWith(originOf(mode)));
          if (bName === 'chromium' && named) {
            rec.focus = await focusPass(page);
            rec.hover = await hoverPass(page);
            rec.textSpacing = await textSpacing(page);
          }
          if (RUN_AXE && (bName === 'chromium' ? named : true)) {
            rec.axeWcag = await runAxe(page, { type: 'tag', values: WCAG_TAGS });
            if (named || bName !== 'chromium') rec.axeBestPractice = await runAxe(page, { type: 'tag', values: ['best-practice'] });
          }
          if (SHOTS && bName === 'chromium' && named && mode === (opt.shotMode || 'file')) {
            const stem = pageName === 'index.html' ? '' : pageName.replace(/\.html?$/, '') + '-';
            await page.evaluate(() => window.scrollTo(0, 0));
            await page.screenshot({ path: path.join(opt.out, `${opt.prefix}-${stem}${vp.name}-${vp.width}x${vp.height}-${theme}-viewport.png`), animations: 'disabled' });
            await page.screenshot({ path: path.join(opt.out, `${opt.prefix}-${stem}${vp.name}-${vp.width}x${vp.height}-${theme}-full.png`), fullPage: true, animations: 'disabled' });
          }
        } catch (e) { rec.error = String(e.message).split('\n')[0].slice(0, 300); }
        rec.ms = Date.now() - t0;
        rec.fonts = await page.evaluate(() => { const m = (fam, text, weight = 400) => { const s = document.createElement('span'); s.style.cssText = `position:absolute;left:-9999px;top:0;font-size:100px;white-space:pre;font-weight:${weight}`; s.style.fontFamily = fam; s.textContent = text; document.body.appendChild(s); const w = s.getBoundingClientRect().width; s.remove(); return Math.round(w * 10) / 10; }; const table = {}; for (const f of ['system-ui', 'ui-monospace', 'monospace', 'sans-serif', 'Selawik', 'Inter', 'DejaVu Sans', 'JetBrains Mono', 'Liberation Sans', 'WenQuanYi Zen Hei']) table[f] = { sample: m(`"${f}"`, 'plyrune 0123 IMWM'), i10: m(`"${f}"`, 'iiiiiiiiii'), M10: m(`"${f}"`, 'MMMMMMMMMM') }; const resolved = (fam) => { for (const [name, v] of Object.entries(table)) if (name !== fam && v.sample === table[fam].sample && v.i10 === table[fam].i10) return name; return 'unidentified'; }; return { table, systemUiResembles: resolved('system-ui'), uiMonospaceResembles: resolved('ui-monospace'), uiMonoIsMonospace: table['ui-monospace'].i10 === table['ui-monospace'].M10, semiboldDistinctFromBold: m('"system-ui"', 'HHHH', 600) !== m('"system-ui"', 'HHHH', 700), semiboldDistinctFromRegular: m('"system-ui"', 'HHHH', 600) !== m('"system-ui"', 'HHHH', 400) }; }).catch((e) => ({ error: String(e.message) }));
        report.runs.push(rec);
        await ctx.close();
      }
    }
    // no-JS rendering + JS interactions + reduced motion + forced colours (Chromium, first mode)
    const mode = modes[0];
    try {
    for (const pageName of pages) {
      if (!fs.existsSync(path.join(root, pageName))) continue;
      const url = baseFor(mode, pageName);
      for (const js of [false, true]) {
        const ctx = await browser.newContext({ viewport: { width: NAMED[0].width, height: NAMED[0].height }, colorScheme: 'dark', javaScriptEnabled: js });
        const page = await ctx.newPage();
        try {
          await page.goto(url, { waitUntil: 'load' });
          await page.evaluate(() => document.fonts.ready);
          const r = await page.evaluate(LAYOUT_PROBE);
          const texts = await page.evaluate(() => ({ h1: [...document.querySelectorAll('h1')].map((h) => h.textContent.trim()), nav: [...document.querySelectorAll('nav a')].map((a) => a.getAttribute('href')), toggleVisible: [...document.querySelectorAll('button')].filter((b) => /theme/i.test(b.textContent + b.getAttribute('aria-label')) && b.getBoundingClientRect().width > 0).length, bodyChars: (document.body.innerText || '').length }));
          report.jsStates = report.jsStates || [];
          report.jsStates.push({ page: pageName, javaScriptEnabled: js, colorScheme: 'dark', effectiveTheme: r.effectiveTheme, ...texts, h1Visible: r.headings.filter((h) => h.startsWith('h1')).length });
          if (js) { report.interactions = report.interactions || []; report.interactions.push({ page: pageName, toggle: await toggleTest(page), disclosure: await disclosureTest(page) }); }
        } catch (e) { report.jsStates = report.jsStates || []; report.jsStates.push({ page: pageName, javaScriptEnabled: js, error: String(e.message).split('\n')[0].slice(0, 200) }); }
        await ctx.close();
      }
      for (const vp of [NAMED[0], NAMED[2]]) {
        const ctx = await browser.newContext({ viewport: { width: vp.width, height: vp.height }, colorScheme: 'light', reducedMotion: 'reduce', isMobile: vp.mobile, hasTouch: vp.mobile });
        const page = await ctx.newPage();
        await page.goto(url, { waitUntil: 'load' });
        const r = await page.evaluate(() => { const dur = (v) => Math.max(0, ...v.split(',').map((x) => { const mm = x.trim().match(/([\d.]+)(ms|s)/); return mm ? parseFloat(mm[1]) * (mm[2] === 's' ? 1000 : 1) : 0; })); const moving = [...document.querySelectorAll('*')].filter((el) => { const cs = getComputedStyle(el); return (cs.animationName !== 'none' && dur(cs.animationDuration) > 0) || (cs.transitionProperty !== 'none' && dur(cs.transitionDuration) > 0); }).map((el) => el.tagName.toLowerCase() + '.' + String(el.className).split(' ')[0]).slice(0, 12); return { scrollBehavior: getComputedStyle(document.documentElement).scrollBehavior, movingElements: moving }; });
        report.reducedMotion = report.reducedMotion || [];
        report.reducedMotion.push({ page: pageName, viewport: vp.name, ...r });
        await ctx.close();
      }
      if (bName === 'chromium') {
        const ctx = await browser.newContext({ viewport: { width: NAMED[0].width, height: NAMED[0].height }, colorScheme: 'light' });
        const page = await ctx.newPage();
        await page.goto(url, { waitUntil: 'load' });
        await page.emulateMedia({ forcedColors: 'active' });
        await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
        const fc = await page.evaluate(() => { const links = [...document.querySelectorAll('a[href]')]; const buttons = [...document.querySelectorAll('button,summary')]; return { forced: matchMedia('(forced-colors: active)').matches, links: links.length, linksWithoutUnderlineOrBorder: links.filter((a) => { const cs = getComputedStyle(a); return !/underline/.test(cs.textDecorationLine) && parseFloat(cs.borderTopWidth) === 0 && parseFloat(cs.borderBottomWidth) === 0 && parseFloat(cs.borderLeftWidth) === 0; }).map((a) => (a.textContent || '').trim().slice(0, 30)).slice(0, 12), buttonsWithoutBorder: buttons.filter((b) => { const cs = getComputedStyle(b); return parseFloat(cs.borderTopWidth) === 0 && parseFloat(cs.borderBottomWidth) === 0; }).length, scrollW: document.documentElement.scrollWidth, vw: document.documentElement.clientWidth }; });
        if (SHOTS && mode === (opt.shotMode || 'file')) await page.screenshot({ path: path.join(opt.out, `${opt.prefix}-${pageName.replace('.html', '')}-desktop-forced-colors-viewport.png`) });
        report.forcedColors = report.forcedColors || [];
        report.forcedColors.push({ page: pageName, ...fc });
        await ctx.close();

        // Print stylesheet, forced together with a dark colour scheme: print must come out light.
        const ctxp = await browser.newContext({ viewport: { width: 816, height: 1056 } });
        const pagep = await ctxp.newPage();
        await pagep.goto(url, { waitUntil: 'load' });
        await pagep.emulateMedia({ media: 'print', colorScheme: 'dark' });
        const pr = await pagep.evaluate(LAYOUT_PROBE);
        const pv = await pagep.evaluate(() => { const d = (sel) => { const el = document.querySelector(sel); return el ? getComputedStyle(el).display : 'absent'; }; return { skip: d('.skip'), nav: d('.site-nav'), toggle: d('.theme-toggle'), actions: d('.actions'), toggleHiddenAttr: document.querySelector('[data-theme-toggle]') ? document.querySelector('[data-theme-toggle]').hidden : null }; });
        if (SHOTS && mode === (opt.shotMode || 'file')) await pagep.screenshot({ path: path.join(opt.out, `${opt.prefix}-${pageName.replace('.html', '')}-print-a4-top.png`) });
        report.print = report.print || [];
        report.print.push({ page: pageName, media: 'print', colorScheme: 'dark', bodyBg: pr.bodyBg, fg: pr.fg, scrollW: pr.scrollW, vw: pr.vw, overflowers: pr.overflowers, clipped: pr.clipped, ...pv });
        await ctxp.close();
      }
    }
    } catch (e) { report.toolErrors = (report.toolErrors || []).concat(`${bName}: optional stage failed: ${String(e.message).split("\n")[0].slice(0, 160)}`); }
    await browser.close().catch(() => {});
  }

  // live 404 behaviour
  if (opt.url) {
    try { const r = await fetch(String(opt.url).replace(/\/$/, '') + '/__plyrune-404-probe__/no-such-page', { redirect: 'manual' }); const t = await r.text(); report.live404 = { status: r.status, headers: Object.fromEntries([...r.headers].filter(([k]) => /content-type|content-security|cache-control|content-encoding|^server$/.test(k))), bodyStart: t.slice(0, 200).replace(/\s+/g, ' ') }; } catch (e) { report.live404 = { error: String(e.message) }; }
  }

  writeSummary();
  for (const s of Object.values(servers)) await s.close();
}

// Written even when a browser or a stage fails, so partial results are never lost.
function writeSummary() {
  if (summaryWritten) return;
  summaryWritten = true;
  const S = (report.summary = report.summary || {});
  try {
  // ---- summary (real numbers only; empty lists mean no finding, not "verified by a human")
  const S = report.summary;
  const runs = report.runs;
  const where = (r) => `${r.browser}/${r.mode}/${r.viewport}/${r.theme}`;
  S.runs = runs.length; S.errors = runs.filter((r) => r.error).map((r) => `${where(r)}: ${r.error}`);
  S.pageErrors = runs.filter((r) => r.pageErrors.length).map((r) => `${where(r)}: ${r.pageErrors.join(' | ')}`);
  const load = (key) => runs.filter((r) => r.load && r.load[key].length).map((r) => `${where(r)}: ${r.load[key].join(' | ')}`);
  S.loadConsole = load('console'); S.loadFailedRequests = load('failedRequests'); S.loadBadResponses = load('badResponses'); S.loadPageErrors = load('pageErrors');
  // Recorded after axe-core was injected and screenshots were taken: tooling noise, not the page's own.
  S.afterToolingConsole = runs.filter((r) => r.console.length).map((r) => `${where(r)}: ${r.console.join(' | ')}`);
  S.afterToolingFailedRequests = runs.filter((r) => r.failedRequests.length).map((r) => `${where(r)}: ${r.failedRequests.join(' | ')}`);
  S.afterToolingBadResponses = runs.filter((r) => r.badResponses.length).map((r) => `${where(r)}: ${r.badResponses.join(' | ')}`);
  S.externalRequests = runs.filter((r) => r.externalRequests && r.externalRequests.length).map((r) => `${where(r)}: ${r.externalRequests.join(' | ')}`);
  S.requestCounts = [...new Set(runs.filter((r) => r.browser === 'chromium').map((r) => `${where(r)}: ${r.load.requests.length} request(s) at load`))];
  S.loadRequestUrls = runs.filter((r) => r.browser === 'chromium' && r.viewport.startsWith('desktop') && r.theme === 'light').map((r) => `${r.page}: ${r.load.requests.map((u) => u.replace(/^file:\/\/\/site\//, '').slice(0, 100)).join(', ')}`);
  S.themeMismatch = runs.filter((r) => r.themeMismatch).map((r) => `${where(r)}: ${r.themeMismatch}`);
  S.effectiveThemes = [...new Set(runs.map((r) => `${where(r)} -> ${r.layout?.effectiveTheme} (bg ${r.layout?.bodyBg})`))];
  S.horizontalScroll = runs.filter((r) => r.layout && r.layout.scrollW > r.layout.vw).map((r) => `${where(r)}: scrollWidth ${r.layout.scrollW} > ${r.layout.vw}`);
  S.overflowers = runs.filter((r) => r.layout && r.layout.overflowers.length).map((r) => `${where(r)}: ${r.layout.overflowers.slice(0, 6).join(' | ')}`);
  S.clipped = runs.filter((r) => r.layout && r.layout.clipped.length).map((r) => `${where(r)}: ${r.layout.clipped.slice(0, 6).join(' | ')}`);
  S.textOverlaps = runs.filter((r) => r.layout && r.layout.textOverlaps.length).map((r) => `${where(r)}: ${r.layout.textOverlaps.slice(0, 6).join(' | ')}`);
  S.zeroSizeTargets = runs.filter((r) => r.layout && r.layout.zeroSizeTargets.length).map((r) => `${where(r)}: ${r.layout.zeroSizeTargets.join(' | ')}`);
  const namedRuns = runs.filter((r) => NAMED.some((n) => r.viewport.startsWith(n.name)));
  const tgt = namedRuns.filter((r) => r.layout).flatMap((r) => r.layout.targets.filter((t) => !t.inline && !t.offscreen).map((t) => ({ ...t, where: where(r) })));
  S.targetsMeasured = tgt.length;
  const uniq = (list) => [...new Map(list.map((t) => [t.sig, t])).values()];
  S.targetsDistinct = uniq(tgt).length;
  S.targetsBelow44 = uniq(tgt.filter((t) => t.w < 44 || t.h < 44)).map((t) => `${t.sig} ${t.w}x${t.h}`);
  S.targetsBelow24 = uniq(tgt.filter((t) => t.w < 24 || t.h < 24)).map((t) => `${t.sig} ${t.w}x${t.h}`);
  S.smallestTargets = uniq(tgt).sort((a, b) => Math.min(a.w, a.h) - Math.min(b.w, b.h)).slice(0, 6).map((t) => `${t.sig} ${t.w}x${t.h}`);
  S.inlineTargets = uniq(namedRuns.filter((r) => r.layout).flatMap((r) => r.layout.targets.filter((t) => t.inline && !t.offscreen).map((t) => ({ ...t, where: r.viewport })))).map((t) => `${t.sig} ${t.w}x${t.h} (inline text link)`);
  const focus = runs.filter((r) => r.focus).flatMap((r) => r.focus.map((f) => ({ ...f, where: `${r.mode}/${r.viewport}/${r.theme}` })));
  S.focusable = [...new Set(focus.map((f) => f.name))];
  S.focusStops = focus.length;
  S.focusWithoutVisibleIndicator = focus.filter((f) => !(f.outlineStyle !== 'none' && (f.outlineStyle === 'auto' || parseFloat(f.outlineWidth) >= 2)) && f.shadow === 'none').map((f) => `${f.where} ${f.name} outline ${f.outlineStyle} ${f.outlineWidth}`);
  S.focusOutlineSamples = [...new Set(focus.map((f) => `${f.outlineStyle} ${f.outlineWidth} ${f.outlineColor}`))];
  S.focusNotVisiblePseudo = focus.filter((f) => !f.focusVisible).map((f) => `${f.where} ${f.name}`);
  S.focusObscured = focus.filter((f) => f.obscured).map((f) => `${f.where} ${f.name}`);
  S.focusOutOfView = focus.filter((f) => !f.inView).map((f) => `${f.where} ${f.name}`);
  S.focusContrastViolations = focus.filter((f) => f.contrastViolations && f.contrastViolations.length).map((f) => `${f.where} ${f.name}: ${f.contrastViolations.join(' | ')}`);
  const hov = runs.flatMap((r) => (r.hover || []).filter((h) => !h.skipped).map((h) => ({ ...h, where: `${r.mode}/${r.viewport}/${r.theme}` })));
  S.hoverStops = hov.length;
  S.hoverContrastViolations = hov.filter((h) => h.contrastViolations && h.contrastViolations.length).map((h) => `${h.where} ${h.name}: ${h.contrastViolations.join(' | ')}`);
  S.hoverErrors = hov.filter((h) => h.error).map((h) => `${h.where} ${h.name}: ${h.error}`);
  S.hoverStyles = [...new Set(hov.filter((h) => h.where.startsWith('file/desktop')).map((h) => `${h.name}: ${h.color} on ${h.bg}, ${h.deco}, outline ${h.outline}`))];
  S.textSpacingProblems = runs.filter((r) => r.textSpacing && (r.textSpacing.overflowers.length || r.textSpacing.clipped.length || r.textSpacing.textOverlaps.length || r.textSpacing.scrollW > r.textSpacing.vw)).map((r) => `${where(r)}: scrollW ${r.textSpacing.scrollW}/${r.textSpacing.vw}; ${[...r.textSpacing.overflowers, ...r.textSpacing.clipped, ...r.textSpacing.textOverlaps].slice(0, 6).join(' | ')}`);
  const axeRuns = runs.filter((r) => r.axeWcag);
  S.axeWcagRuns = axeRuns.length;
  S.axeWcagViolations = axeRuns.flatMap((r) => r.axeWcag.violations.map((v) => `${where(r)}: ${v.id} (${v.impact}) x${v.nodes.length} ${v.nodes.slice(0, 3).map((n) => n.target + ' ' + (n.any.join('; ') || n.summary)).join(' || ')}`));
  S.axeWcagIncomplete = axeRuns.flatMap((r) => r.axeWcag.incomplete.map((v) => `${where(r)}: ${v.id} x${v.nodes.length} ${v.nodes.slice(0, 3).map((n) => n.target).join(', ')}`));
  const first = axeRuns.find((r) => r.browser === 'chromium');
  S.axeRulesRunWcag = first ? [...new Set([...first.axeWcag.violations, ...first.axeWcag.incomplete, ...first.axeWcag.passes].map((v) => v.id))].sort() : [];
  S.axeRulesRunWcag22 = first ? [...new Set([...first.axeWcag.violations, ...first.axeWcag.incomplete, ...first.axeWcag.passes].filter((v) => v.tags.includes('wcag22aa')).map((v) => v.id))].sort() : [];
  S.axePassCounts = first ? Object.fromEntries([...first.axeWcag.passes.map((p) => [p.id, p.n]), ...first.axeWcag.violations.map((p) => [p.id, 0]), ...first.axeWcag.incomplete.map((p) => [p.id, 0])]) : {};
  const bp = runs.filter((r) => r.axeBestPractice);
  S.axeBestPracticeRuns = bp.length;
  S.axeBestPracticeViolations = bp.flatMap((r) => r.axeBestPractice.violations.map((v) => `${where(r)}: ${v.id} (${v.impact}) x${v.nodes.length} ${v.nodes.slice(0, 3).map((n) => n.target + ' ' + (n.any.join('; ') || n.summary)).join(' || ')}`));
  S.axeBestPracticeIncomplete = bp.flatMap((r) => r.axeBestPractice.incomplete.map((v) => `${where(r)}: ${v.id} x${v.nodes.length}`));
  const ch = runs.find((r) => r.refs);
  if (ch) { S.refVerdicts = Object.fromEntries(Object.entries(ch.refs.reduce((a, r) => { a[r.verdict] = (a[r.verdict] || 0) + 1; return a; }, {})).sort()); S.nonRelativeRefs = ch.refs.filter((r) => !['relative', 'fragment'].includes(r.verdict)); S.fragmentTargetsMissing = [...new Set(ch.refs.filter((r) => r.verdict === 'fragment' && r.ref.length > 1 && !ch.ids.includes(decodeURIComponent(r.ref.slice(1)))).map((r) => r.ref))]; S.refsMeta = ch.meta; S.cssUrls = (report.cssUrls || []).map((x) => x[2]); }
  S.fragmentAnchors = ch ? [...new Set(ch.refs.filter((r) => r.verdict === 'fragment' && r.ref.length > 1).map((r) => r.ref))] : [];
  S.wording = wording.map((w) => ({ page: w.label, missing: w.missing, forbidden: w.forbidden, watch: w.watch }));
  S.weights = weights;
  S.fontProbes = runs.filter((r) => r.fonts && r.fonts.table).slice(0, 1).map((r) => JSON.stringify(r.fonts));
  S.pageHeights = Object.fromEntries(runs.filter((r) => r.layout && NAMED.some((n) => r.viewport.startsWith(n.name)) && r.browser === 'chromium' && r.mode === modes[0]).map((r) => [`${r.page} ${r.viewport} ${r.theme}`, r.layout.pageH]));
  S.landmarks = ch ? ch.layout.landmarks : null;
  S.headings = ch ? ch.layout.headings : null;
  S.tokens = runs.filter((r) => r.tokens).map((r) => ({ where: where(r), pairs: r.tokens.filter((t) => t.ratio !== null).map((t) => `${t.fg} on ${t.bg} = ${t.ratio}${t.ratio >= 4.5 ? ' (AA normal text)' : t.ratio >= 3 ? ' (AA large text/UI only)' : ' (FAIL)'}`), notDefined: r.tokens.filter((t) => t.ratio === null).map((t) => `${t.fg}/${t.bg}`) }));
  const tok = runs.filter((r) => r.tokens).flatMap((r) => r.tokens.filter((t) => t.ratio !== null).map((t) => ({ ratio: t.ratio, pair: `${t.fg}/${t.bg}`, where: where(r) })));
  S.tokenMin = tok.length ? tok.reduce((m, t) => (t.ratio < m.ratio ? t : m)) : null;
  S.axeCatalog = report.axeCatalog ? { total: report.axeCatalog.length, enabled: report.axeCatalog.filter((r) => r.enabled).length, wcag22: report.axeCatalog.filter((r) => r.tags.some((t) => /^wcag22/.test(t))).map((r) => `${r.id} [${r.tags.filter((t) => /wcag\d/.test(t)).join(',')}]${r.enabled ? '' : ' (disabled by default)'}`) } : null;
  S.jsStates = report.jsStates; S.interactions = report.interactions;
  S.reducedMotion = report.reducedMotion; S.forcedColors = report.forcedColors; S.print = report.print || [];
  if (opt.url) { S.liveDocHeaders = runs.find((r) => r.docHeaders && r.page === 'index.html')?.docHeaders; S.liveResponses = runs.find((r) => r.responses)?.responses; S.live404 = report.live404; S.liveStatuses = [...new Set(runs.map((r) => `${where(r)}: HTTP ${r.status}`))]; }
  const srv = servers.subpath || servers.root;
  if (srv) { const gets = srv.log.filter((l) => l.method === 'GET'); S.httpRequests = [...new Set(gets.map((l) => `${l.status} ${l.path} (${l.bytes} B${l.type ? ', ' + l.type.split(';')[0] : ''})`))]; S.httpBytesServed = gets.reduce((s, l) => s + l.bytes, 0); S.httpNot200 = gets.filter((l) => l.status !== 200).map((l) => `${l.status} ${l.path}`); }
  fs.writeFileSync(path.join(opt.out, `${opt.prefix}-report.json`), JSON.stringify(report, null, 1));
  } catch (e) { S.summaryError = String(e.message).slice(0, 200); }
  if (report.toolErrors) S.toolErrors = report.toolErrors;
  try { fs.mkdirSync(opt.out, { recursive: true }); fs.writeFileSync(path.join(opt.out, `${opt.prefix}-report.json`), JSON.stringify(report, null, 1)); } catch (e) { console.error("could not write report", String(e.message)); }
  console.log(JSON.stringify(S, null, 1));
}
main().catch((e) => { console.error("FATAL", String((e && e.message) || e).slice(0, 300)); writeSummary(); process.exit(1); });
