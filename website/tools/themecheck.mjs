// In the pinned Playwright image: node themecheck.mjs /site (staged tree), or pass the live URL.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { extname, join } from 'node:path';
import { chromium } from 'playwright-core';

const target = process.argv[2] || '/site';
const missing = 'not-a-plyrune-page/nested/missing';
const types = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.svg': 'image/svg+xml', '.ico': 'image/x-icon', '.png': 'image/png' };
let server;
async function siteURL() {
  if (/^https?:\/\//.test(target)) return new URL(target);
  const errorPage = await readFile(join(target, '404.html'));
  server = createServer(async (req, res) => {
    const file = new URL(req.url, 'http://localhost').pathname.slice(1) || 'index.html';
    try {
      const body = await readFile(join(target, file));
      res.writeHead(200, { 'content-type': types[extname(file)] || 'application/octet-stream' }).end(body);
    } catch {
      res.writeHead(404, { 'content-type': 'text/html' }).end(errorPage);
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return new URL(`http://127.0.0.1:${server.address().port}/`);
}

async function palette(page, theme) {
  await page.waitForFunction(expected => getComputedStyle(document.documentElement).colorScheme === expected, theme);
  const background = await page.locator('body').evaluate(body => getComputedStyle(body).backgroundColor);
  assert.equal(background, theme === 'dark' ? 'rgb(17, 31, 27)' : 'rgb(245, 242, 234)', page.url());
}
const switchTo = (page, theme) => page.getByRole('button', { name: `Switch to ${theme} theme`, exact: true });

let browser;
try {
  const base = await siteURL();
  browser = await chromium.launch();
  const prefs = await browser.newContext({ baseURL: base.href, colorScheme: 'dark' });
  // Capture at the parser's interactive transition, before DOMContentLoaded (where controls bind).
  await prefs.addInitScript(() => document.addEventListener('readystatechange', () => {
    if (document.readyState === 'interactive') window.themeAtParse = document.documentElement.dataset.theme;
  }));
  const home = await prefs.newPage();
  const notes = await prefs.newPage();
  await home.goto('./');
  await notes.goto('changelog.html');
  await palette(home, 'dark');
  await home.emulateMedia({ colorScheme: 'light' });
  await palette(home, 'light');
  await home.emulateMedia({ colorScheme: 'dark' });
  await switchTo(home, 'light').press('Enter');
  await palette(home, 'light');
  await palette(notes, 'light');
  await notes.reload();
  assert.equal(await notes.evaluate(() => window.themeAtParse), 'light', 'saved choice before controls bind');
  await palette(notes, 'light');

  const response = await home.goto(missing);
  assert.equal(response.status(), 404);
  assert.equal(await home.evaluate(() => window.themeAtParse), 'light', 'nested 404 reads the same preference');
  await palette(home, 'light');
  await switchTo(home, 'dark').press('Space');
  await palette(home, 'dark');
  await palette(notes, 'dark');
  await home.getByRole('link', { name: /Plyrune home/i }).click();
  assert.equal(new URL(home.url()).pathname, base.pathname, '404 home link resolves at the site root');
  await palette(home, 'dark');
  await notes.emulateMedia({ colorScheme: 'light' });
  await palette(notes, 'dark'); // explicit preference wins even as the OS changes
  await home.evaluate(() => localStorage.clear());
  await palette(notes, 'light'); // another tab cleared site data; system choice resumes
  await prefs.close();

  const denied = await browser.newContext({ baseURL: base.href, colorScheme: 'light' });
  await denied.addInitScript(() => Object.defineProperty(window, 'localStorage', {
    get() { throw new DOMException('Unavailable', 'SecurityError'); }
  }));
  const privatePage = await denied.newPage();
  await privatePage.goto('changelog.html');
  await switchTo(privatePage, 'dark').press('Enter');
  await palette(privatePage, 'dark');
  await privatePage.reload();
  await palette(privatePage, 'light');
  await denied.close();

  const disabled = await browser.newContext({ baseURL: base.href, colorScheme: 'dark', javaScriptEnabled: false });
  const plain = await disabled.newPage();
  for (const path of ['./', 'changelog.html', missing]) {
    await plain.goto(path);
    await palette(plain, 'dark');
    assert.equal(await plain.locator('[data-theme-toggle]').isVisible(), false);
    assert.equal(await plain.getByRole('heading', { level: 1 }).isVisible(), true);
  }
  await disabled.close();
  console.log('Theme preference: system, keyboard override, cross-tab, reload, nested 404, denied storage and no-JS passed.');
} finally {
  try { await browser?.close(); } finally { server?.close(); }
}
