#!/usr/bin/env node
// Dev-only: prints the key facts from a sitecheck report as plain lines (for DESIGN.md and reports).
// Usage: node summarize.mjs /out/DIR/PREFIX-report.json
import fs from 'node:fs';

const file = process.argv[2];
if (!file) { console.error('usage: node summarize.mjs REPORT.json'); process.exit(2); }
const r = JSON.parse(fs.readFileSync(file, 'utf8'));
const S = r.summary || {};
const idx = r.runs.filter((x) => x.browser === 'chromium' && x.page === 'index.html');
const out = [];
out.push(`report: ${file}`);
out.push(`generated: ${r.generatedAt} | node ${r.node} | chromium ${r.chromiumVersion || '-'} | firefox ${r.firefoxVersion || '-'} | webkit ${r.webkitVersion || '-'}`);
out.push(`fonts: ${r.fontNote}`);
out.push(`runs: ${S.runs} (pages: ${[...new Set(r.runs.map((x) => x.page))].join(', ')}; modes: ${[...new Set(r.runs.map((x) => x.mode))].join(', ')}; browsers: ${[...new Set(r.runs.map((x) => x.browser))].join(', ')})`);
const empty = Object.entries(S).filter(([k, v]) => Array.isArray(v) && v.length === 0).map(([k]) => k);
const nonEmpty = Object.entries(S).filter(([k, v]) => Array.isArray(v) && v.length > 0).map(([k, v]) => `${k}(${v.length})`);
out.push(`empty lists: ${empty.join(', ')}`);
out.push(`non-empty lists: ${nonEmpty.join(', ') || 'none'}`);
const first = r.runs.find((x) => x.axeWcag && x.browser === 'chromium' && x.page === 'index.html');
const bp = r.runs.find((x) => x.axeBestPractice && x.browser === 'chromium' && x.page === 'index.html');
if (first) {
  const wc = [...new Set([...first.axeWcag.passes, ...first.axeWcag.violations, ...first.axeWcag.incomplete].map((v) => v.id))];
  const bpr = bp ? [...new Set([...bp.axeBestPractice.passes, ...bp.axeBestPractice.violations, ...bp.axeBestPractice.incomplete].map((v) => v.id))] : [];
  out.push(`axe rules that ran on index.html (desktop/light): ${wc.length} WCAG-tagged + ${bpr.length} best-practice-tagged = ${new Set([...wc, ...bpr]).size} distinct; nodes passing: ${first.axeWcag.passes.reduce((s, p) => s + p.n, 0)} WCAG, ${bp ? bp.axeBestPractice.passes.reduce((s, p) => s + p.n, 0) : 0} best-practice`);
  out.push(`axe WCAG 2.2-tagged rules that ran: ${(S.axeRulesRunWcag22 || []).join(', ') || 'none'}`);
  out.push(`axe WCAG-tagged rule ids: ${(S.axeRulesRunWcag || []).join(", ")}`);
  out.push(`axe best-practice-tagged rule ids: ${bpr.join(", ")}`);
  if (S.axeCatalog) out.push(`axe catalogue: ${S.axeCatalog.total} rules, ${S.axeCatalog.enabled} enabled by default; wcag22 tags: ${S.axeCatalog.wcag22.join(' ; ')}`);
  out.push(`axe WCAG runs: ${S.axeWcagRuns}, violations ${S.axeWcagViolations.length}, incomplete ${S.axeWcagIncomplete.length} | best-practice runs: ${S.axeBestPracticeRuns}, violations ${S.axeBestPracticeViolations.length}, incomplete ${S.axeBestPracticeIncomplete.length}`);
}
if (idx.length) {
  const withRefs = idx.find((x) => x.refs);
  if (withRefs) { out.push(`references: ${JSON.stringify(S.refVerdicts)} -> ${withRefs.refs.map((x) => x.ref).join(', ')}`); out.push(`document: title "${withRefs.meta.docTitle}", lang ${withRefs.meta.lang}, viewport "${withRefs.meta.viewportMeta}", favicons ${withRefs.meta.faviconLinks.join(', ')}, inline scripts ${withRefs.meta.inlineScripts}, external scripts ${withRefs.meta.externalScripts.length}, inline style attributes ${withRefs.refs.filter((x) => x.kind === 'style-attr').length}, event handler attributes ${withRefs.meta.inlineEventHandlers}, embeds ${withRefs.meta.embeds}, meta refresh ${withRefs.meta.metaRefresh}, svg without name ${withRefs.meta.svgsWithoutName}`); }
  out.push(`targets (distinct, index.html): ${[...new Map(idx.flatMap((x) => x.layout.targets.filter((t) => !t.offscreen).map((t) => [t.sig, t]))).values()].map((t) => `${t.sig} ${t.w}x${t.h}`).join(' | ')}`);
  out.push(`page heights (CSS px): ${idx.filter((x) => x.mode === (r.modes[0] === 'live' ? 'live' : 'file')).map((x) => `${x.viewport.split(' ')[0]}/${x.theme}: ${x.layout.pageH}`).join(', ')}`);
  out.push(`viewports checked: ${[...new Set(r.runs.filter((x) => x.browser === 'chromium').map((x) => x.viewport.split(' ')[1]))].join(', ')}`);
}
out.push(`focus: ${S.focusStops} stops, ${S.focusable.length} distinct focusable elements, outline ${[...new Set(S.focusOutlineSamples)].join(' / ')}`);
out.push(`hover: ${S.hoverStops} stops, contrast violations ${S.hoverContrastViolations.length}`);
out.push(`js states: ${JSON.stringify((S.jsStates || []).map((j) => ({ page: j.page, js: j.javaScriptEnabled, system: j.colorScheme, rendered: j.effectiveTheme, toggleShown: j.toggleVisible, h1: j.h1Visible })))}`);
out.push(`interactions: ${JSON.stringify((S.interactions || []).map((i) => ({ page: i.page, toggleFlipsBothWays: i.toggle.flipsBothWays, disclosureToggles: i.disclosure.togglesByClick && i.disclosure.togglesByEnter && i.disclosure.togglesBySpace })))}`);
out.push(`reduced motion: ${JSON.stringify((S.reducedMotion || []).map((m) => ({ page: m.page, viewport: m.viewport, scrollBehavior: m.scrollBehavior, moving: m.movingElements.length })))}`);
out.push(`forced colours: ${JSON.stringify((S.forcedColors || []).map((f) => ({ page: f.page, links: f.links, linksWithoutUnderlineOrBorder: f.linksWithoutUnderlineOrBorder.length, buttonsWithoutBorder: f.buttonsWithoutBorder, scrollW: f.scrollW, vw: f.vw })))}`);
out.push(`print: ${JSON.stringify((S.print || []).map((p) => ({ page: p.page, system: p.colorScheme, bodyBg: p.bodyBg, skip: p.skip, nav: p.nav, toggle: p.toggle, actions: p.actions, overflow: p.overflowers.length })))}`);
if (S.tokens) for (const t of S.tokens.filter((x) => x.where.includes('desktop'))) out.push(`tokens ${t.where}: ${t.pairs.join(' · ')}`);
if (S.weights) out.push(`weights: ${JSON.stringify(Object.fromEntries(Object.entries(S.weights.files).map(([k, v]) => [k, `${v.raw} B${v.brotli ? ` (${v.brotli} B brotli, ${v.gzip} B gzip)` : ''}`])))} | html+css+js ${S.weights.htmlCssJsRaw} B (${S.weights.htmlCssJsBrotli} B brotli) | all ${S.weights.allRaw} B | .nojekyll ${S.weights.nojekyll} | CNAME ${S.weights.cname}`);
if (S.liveDocHeaders) out.push(`live document headers: ${JSON.stringify(S.liveDocHeaders)}`);
if (S.liveResponses) out.push(`live responses: ${JSON.stringify(S.liveResponses)}`);
if (S.live404) out.push(`live 404 probe: ${JSON.stringify({ status: S.live404.status, headers: S.live404.headers })}`);
if (S.httpRequests) out.push(`local http requests: ${S.httpRequests.join(' ; ')} | not-200: ${S.httpNot200.length}`);
out.push(`wording: ${JSON.stringify((S.wording || []).map((w) => ({ page: w.page, missingRequired: w.missing, forbidden: w.forbidden, watch: Object.fromEntries(Object.entries(w.watch).map(([k, v]) => [k, v.length])) })))}`);
console.log(out.join('\n'));
