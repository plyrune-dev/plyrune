#!/usr/bin/env node
// Generates the Plyrune brand SVGs from hand-defined geometry. Development-only; the outputs in
// website/brand/ are committed, so the site itself has no build step.
//
// Usage: node make-brand.mjs OUT_DIR        (default: /out/brand)
//
// Geometry is original, authored for Plyrune on 23 November 2026 from measurements of the
// preferred "01 / JOIN" concept on website/assets/identity-studies.png (a generated reference
// that was measured and regularised, not traced). Units: a strand is t=10 thick, a gap g=5.
import fs from 'node:fs';
import path from 'node:path';

const out = process.argv[2] || '/out/brand';
fs.mkdirSync(out, { recursive: true });

const INK = '#172923', PAPER = '#F5F2EA', TEAL = '#236A57', TEAL_ON_DARK = '#96D0B8';
const t = 10, g = 5;
const d = t * Math.SQRT2; // horizontal width of a 45° strand
const q = g * Math.SQRT2; // horizontal offset of a parallel 45° gap
const W = 75; // outer right edge of the bowl
const a = 15; // top band, top edge (y)
const cO = 13; // outer chamfer leg
const cI = cO - t * (2 - Math.SQRT2); // inner chamfer leg for a uniform-thickness hook (about 7.142)
const topUpper = a + 15; // top strand upper diagonal: line X+Y
const topLower = topUpper + d;
const tealUpper = topLower + q;
const tealLower = tealUpper + d;
const stemLeft = tealLower + q;
const stemRight = stemLeft + d;
const footL = 15, footR = 25;
const footBottom = stemRight - footR + g;
const f = (n) => (Math.round(n * 1000) / 1000).toString();
const P = (...pts) => pts.map(([x, y], i) => (i ? `L${f(x)} ${f(y)}` : `M${f(x)} ${f(y)}`)).join('') + 'Z';

// Long ink strand: top-left band -> up -> top bar -> chamfered bowl -> bottom bar -> diagonal stem -> foot.
const HOOK = P(
  [topUpper, 0], [W - cO, 0], [W, cO], [W, 40 - cO], [W - cO, 40], [stemLeft - 40, 40], [footL, stemLeft - footL],
  [footL, footBottom], [footR, footBottom], [footR, stemRight - footR], [stemRight - 30, 30], [65 - cI, 30],
  [65, 30 - cI], [65, 10 + cI], [65 - cI, 10], [topLower - 10, 10], [topLower - 25, 25], [0, 25], [0, a], [a, a],
);
// Teal strand: middle band -> up -> middle bar; its right end is closed by the hook's inner chamfers.
const STRAND = P(
  [0, 30], [tealUpper - 30, 30], [tealUpper - 15, 15], [(65 - cI) + 10 - 15, 15], [65, 10 + cI], [65, 30 - cI],
  [65 + 30 - cI - 25, 25], [tealLower - 25, 25], [tealLower - 40, 40], [0, 40],
);
// Bottom-left stub, cut on the teal strand's lower edge line so the diagonal channel stays straight.
const STUB = P([0, 45], [tealLower - 45, 45], [tealLower - 55, 55], [0, 55]);
const MARK_W = W, MARK_H = footBottom;

// ---- small-size optical variant (favicon): the three entering strands do not survive 16 px, so
// only the chamfered bowl, its middle bar and the diagonal stem remain. Every coordinate is even on
// this 32-unit grid, so edges fall on pixel boundaries at 16 px and 32 px (pixel-snapped, not shrunk).
const SMALL_OUTER = P([8, 4], [22, 4], [26, 8], [26, 14], [22, 18], [16, 18], [10, 24], [10, 28], [6, 28], [6, 24], [12, 18], [8, 18]);
const SMALL_HOLE_TOP = P([10, 6], [22, 6], [24, 8], [24, 10], [10, 10]);
const SMALL_HOLE_BOTTOM = P([10, 12], [24, 12], [24, 14], [22, 16], [10, 16]);

// ---- wordmark: original monoline letterforms (no font outlines). x-height 100, stroke 20,
// baseline y=100, ascender -45, descender 140. Round bowls overshoot x-line and baseline by 1.5.
const RX = 40, RY = 41.5; // bowl ellipse of p and e
const ARC_R = 40.03; // u/n: circle through (10,50), the extreme and (90,50); meets the stems tangentially
// y's diagonal is a filled parallelogram: horizontal top cut, tip meeting the stem on the baseline.
const Y_CUT = 20 / Math.sin(Math.atan2(100, 62)); // horizontal width of that stroke, cut flat at the x-line
const letter = {
  p: (x) => [`M${f(x + 10)} 0V140`, `M${f(x + 10)} 50A${RX} ${RY} 0 1 1 ${f(x + 90)} 50A${RX} ${RY} 0 1 1 ${f(x + 10)} 50`],
  l: (x) => [`M${f(x + 10)} -45V100`],
  y: (x) => [`M${f(x)} 0H${f(x + Y_CUT)}L${f(x + 62)} ${f((100 / 62) * Y_CUT)}V100Z`, `M${f(x + 72)} 0V140`],
  r: (x) => [`M${f(x + 10)} 0V100`, `M${f(x + 10)} 42A32 32 0 0 1 ${f(x + 42)} 10H${f(x + 56)}`],
  u: (x) => [`M${f(x + 90)} 0V50A${ARC_R} ${ARC_R} 0 0 1 ${f(x + 10)} 50V0`],
  n: (x) => [`M${f(x + 10)} 0V100`, `M${f(x + 10)} 50A${ARC_R} ${ARC_R} 0 0 1 ${f(x + 90)} 50V100`],
  e: (x) => [`M${f(x + 82.78)} 73.81A${RX} ${RY} 0 1 1 ${f(x + 90)} 50H${f(x + 10)}`],
};
const WIDTH = { p: 100, l: 20, y: 82, r: 56, u: 100, n: 100, e: 100 };
const GAP = { pl: 15, ly: 15, yr: 12, ru: 13, un: 12, ne: 15 };
function wordmark() {
  let x = 0; const parts = { stroke: [], fill: [] };
  'plyrune'.split('').forEach((ch, i) => {
    for (const p of letter[ch](x)) (p.endsWith('Z') ? parts.fill : parts.stroke).push(p);
    const next = 'plyrune'[i + 1];
    if (next) x += WIDTH[ch] + GAP[ch + next];
  });
  return { ...parts, width: x + WIDTH.e };
}
const WM = wordmark();

const svg = (vb, body) => `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${vb}">${body}</svg>\n`;
const TITLE = '<title>Plyrune</title>';
const ICON_VB = '0 0 100 100';
const ICON_T = 'translate(12.5 16.7155)'; // centres the 75 x 66.569 mark in a 100-unit box
const markGroup = (ink, accent, t = ICON_T) => `<g transform="${t}"><path d="${HOOK}" fill="${ink}"/><path d="${STUB}" fill="${ink}"/><path d="${STRAND}" fill="${accent}"/></g>`;
const iconFile = (ink, accent, d) => svg(ICON_VB, `<desc>${d}</desc>${TITLE}${markGroup(ink, accent)}`);

function lockupBody(ink, accent) {
  const s = 0.26; // wordmark scale: x-height 26 beside the 66.6-unit icon; baseline on the icon's centre line
  const strokes = WM.stroke.map((p) => `<path d="${p}"/>`).join('');
  const fills = WM.fill.map((p) => `<path d="${p}"/>`).join('');
  return `<desc>Plyrune: Join mark and lowercase wordmark</desc>${TITLE}${markGroup(ink, accent)}` +
    `<g transform="translate(${f(12.5 + MARK_W + 22)} ${f(16.7155 + MARK_H / 2)}) scale(${s})" fill="none" stroke="${ink}" stroke-width="20" stroke-linecap="butt" stroke-linejoin="miter">${strokes}<g fill="${ink}" stroke="none">${fills}</g></g>`;
}
const LOCKUP_W = 12.5 * 2 + MARK_W + 22 + WM.width * 0.26;

const files = {
  'join-icon.svg': iconFile(INK, INK, 'Join mark, monochrome'),
  'join-icon-reversed.svg': iconFile(PAPER, PAPER, 'Join mark, reversed for dark backgrounds'),
  'join-icon-two-tone.svg': iconFile(INK, TEAL, 'Join mark, two tone'),
  'join-icon-two-tone-reversed.svg': iconFile(PAPER, TEAL_ON_DARK, 'Join mark, two tone, reversed for dark backgrounds'),
  'lockup.svg': svg(`0 0 ${f(LOCKUP_W)} 100`, lockupBody(INK, INK)),
  'lockup-reversed.svg': svg(`0 0 ${f(LOCKUP_W)} 100`, lockupBody(PAPER, PAPER)),
  'lockup-two-tone.svg': svg(`0 0 ${f(LOCKUP_W)} 100`, lockupBody(INK, TEAL)),
  'lockup-two-tone-reversed.svg': svg(`0 0 ${f(LOCKUP_W)} 100`, lockupBody(PAPER, TEAL_ON_DARK)),
  // Favicon: one asset that reads on light and dark tab strips (ink tile, reversed mark, teal bar).
  'favicon.svg': svg('0 0 32 32', `<desc>Plyrune</desc>${TITLE}<rect width="32" height="32" rx="8" fill="${INK}"/><path d="${SMALL_OUTER}" fill="${PAPER}" fill-rule="evenodd"/><path d="${SMALL_HOLE_TOP}${SMALL_HOLE_BOTTOM}" fill="${INK}"/><rect x="10" y="10" width="14" height="2" fill="${TEAL_ON_DARK}"/>`),
  // App / touch icon: the full mark on a tile, for 48 px and above.
  'app-icon.svg': svg('0 0 100 100', `<desc>Plyrune</desc>${TITLE}<rect width="100" height="100" rx="22" fill="${INK}"/>${markGroup(PAPER, TEAL_ON_DARK, 'translate(14 18.2) scale(0.96)')}`),
};
for (const [name, body] of Object.entries(files)) fs.writeFileSync(path.join(out, name), body);
const geometry = { units: { t, g, d, q, cO, cI }, lines: { topUpper, topLower, tealUpper, tealLower, stemLeft, stemRight, footBottom }, mark: { w: MARK_W, h: MARK_H }, wordmark: { width: WM.width, strokes: WM.stroke.length, fills: WM.fill.length }, lockup: { w: LOCKUP_W, h: 100 } };
fs.writeFileSync(path.join(out, 'geometry.json'), JSON.stringify(geometry, null, 1));
console.log(Object.keys(files).map((n) => `${String(fs.statSync(path.join(out, n)).size).padStart(6)}  ${n}`).join('\n'));
console.log('geometry', JSON.stringify(geometry));
