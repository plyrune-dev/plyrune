#!/usr/bin/env node
// Dev-only: sends each HTML file to the W3C Nu Html Checker and prints errors and warnings.
// Needs network access to validator.w3.org. Usage: node htmlcheck.mjs ROOT_FILE_OR_DIR [...]
import fs from 'node:fs';
import path from 'node:path';

const targets = process.argv.slice(2).flatMap((t) => (fs.statSync(t).isDirectory() ? fs.readdirSync(t).filter((f) => f.endsWith('.html')).map((f) => path.join(t, f)) : [t]));
let failed = 0;
for (const file of targets) {
  const body = fs.readFileSync(file);
  const res = await fetch('https://validator.w3.org/nu/?out=json&parser=html', { method: 'POST', headers: { 'Content-Type': 'text/html; charset=utf-8', 'User-Agent': 'plyrune-website-tools/1.0 (local check)' }, body });
  const json = await res.json();
  const msgs = json.messages || [];
  const errors = msgs.filter((m) => m.type === 'error'), warnings = msgs.filter((m) => m.type !== 'error');
  console.log(`${path.basename(file)}: HTTP ${res.status}, ${errors.length} error(s), ${warnings.length} warning(s)`);
  for (const m of msgs) console.log(`  [${m.type}] line ${m.lastLine}:${(m.message || '').replace(/\s+/g, ' ').slice(0, 200)}${m.extract ? ' | ' + m.extract.replace(/\s+/g, ' ').trim().slice(0, 80) : ''}`);
  if (errors.length) failed++;
}
process.exit(failed ? 1 : 0);
