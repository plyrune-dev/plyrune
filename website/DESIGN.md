# Plyrune website: design and delivery record

Live at https://plyrune-dev.github.io/ since 23 November 2026 (first preview) and 24 November 2026 (production build).

The current site is a short guide to the released CLI, its stale-input example and the limits of its proof. It is not a product-roadmap dashboard. The measurements, screenshots and comparisons from section 2 onward describe earlier site revisions, not accessibility or layout results for this release.

## 1. What is where

| Path | Published? | Purpose |
|---|---|---|
| `index.html`, `changelog.html`, `install.sh` | yes | Preview walkthrough, dated source/release notes and checksum-pinned Linux installer |
| `theme.js` | yes | Restores the explicit origin-local light/dark preference before paint; otherwise the system palette applies |
| `styles.css` | yes | All styling, both colour schemes, print and forced-colours rules |
| `404.html` | yes | Not-found page; shared assets resolve at the organization site's root even from a nested missing URL |
| `measurements.json` | yes | Unmeasured task outcomes, linked from the evidence section |
| `.nojekyll` | yes | Tells GitHub Pages to publish the files as they are |
| `brand/favicon.svg`, `brand/favicon.ico`, `brand/apple-touch-icon.png` | yes | The icons the page references |
| `brand/*.svg` (icons and lockups) | no | Production brand vectors, kept for reuse |
| `assets/*.png` | no | Generated concept references (4.9 MB), unchanged from the handover |
| `tools/` | no | Publish allowlist, staging script, headless check runner, brand generators |
| `DESIGN.md`, `PROMPTS.md` | no | This record and the concept prompts |

Exactly the "yes" rows are listed in [tools/publish-allowlist.txt](tools/publish-allowlist.txt). [tools/stage.sh](tools/stage.sh) copies only those paths. Nothing else under `website/` has ever been published.

There is no build step. The files above are the site.

## 2. What changed from the prototype, and why

The prototype was commit `c375f59` (a 13.5 KB single file). The supplied reference `Plyrune-Website-Preview.html` is byte-identical to it (sha256 `f4c3aa5a…a03c`), and the supplied zip is the handover pack, so the prototype is the only visual reference. Everything below was measured on both versions with the same tool and fonts ([tools/sitecheck.mjs](tools/sitecheck.mjs); reports are kept with the screenshots, see section 6).

| Area | Prototype | Now | Why |
|---|---|---|---|
| Structure | one file holding page, CSS and JS, next to 4.9 MB of concept PNGs and private notes | page + stylesheet + 404 + icons, published from an explicit allowlist (8 files, 55,667 B) | a public deploy must not ship concept art or internal notes |
| Brand | text wordmark, no icon, no favicon | vector lockup in the header; favicon in three formats | production asset; the handover required it before launch |
| Colour scheme | light only; dark reachable by a button that does nothing without JavaScript | follows `prefers-color-scheme`, optional per-page override, no storage | a dark-system visitor got the light page and a dead control |
| Target size | three navigation links 21.7 px tall | smallest control 44 px (buttons 48 px) | WCAG 2.2 target size and the brief's 44 px rule |
| Narrow screens | horizontal scroll at 320 px (scrollWidth 342) and under text-spacing at 390 px (411) | none at 320 to 1920 px, in both themes, also under text spacing | reflow |
| Type size | 14 px body, 11 to 12 px labels | 17 px body, 14 px secondary text, 13 px monospace labels and 12 px chip and row labels | readability |
| Flow diagram | four steps, a footnote | four steps, an explicit dashed candidate-only boundary, and a greyed "Apply" step marked as outside M1–M4 | the boundary is the product's central constraint |
| Example fixture | two captures, four expandable | same, plus the requirement, inventory, a solid/dashed capture diagram, expected candidate and check, and the narrowed-scope rule; labelled "Not engine output" and "static example" | show that displaying less never means selecting less |
| Assurance | one list of four claims | available / claimed / measured kept apart; the first composition claim quoted; eight claims with their planned negative controls; inherited evidence scoped separately | the handover requires exact statements and forbids folding them into a badge |
| Measurements | four rows of dashes | explicit empty state: what will be counted, "not measured, never zero", method, link to the data file | an empty state that explains itself |
| Roadmap | three grouped columns, no status | M0–M5 with dated statuses (text and shape, never colour alone) | readers need to know what exists |
| Page weight | 13,573 B | 47,454 B for homepage HTML + CSS + JS | still far below the 150 KB target |

Kept from the prototype because they work: the ivory homepage and dark workbench palette, the headline "Source to change, without losing the thread.", the dark strap, eyebrow labels in monospace capitals, thin rules instead of card grids, tabular numerals, and the two-column hero.

## 3. Design system

Tokens live at the top of [styles.css](styles.css). Light values are the handover's palette; dark values are the prototype's dark theme. New: `--muted` (#4A5A53, darker than the prototype's #506058), `--rule`, `--focus` (copper in both themes) and the code-block colours.

| Pair | Light | Dark |
|---|---|---|
| ink on paper | 13.62 | 15.19 |
| muted on paper | 6.52 | 9.49 |
| muted on panel | 6.12 | 7.82 |
| teal (links, output names) on paper | 5.74 | 9.72 |
| copper (labels, focus ring) on paper | 4.90 | 8.12 |
| copper on panel (lowest pair on the page) | 4.60 | 6.70 |
| on-teal on teal (primary button) | 6.42 | 9.72 |
| code text on code background | 13.62 | 16.61 |

Ratios were computed in the browser from the live custom properties on the rendered page (WCAG relative luminance), not by hand.

Type: the system font stack only (`system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif`; monospace `ui-monospace, "SF Mono", "Cascadia Code", "Segoe UI Mono", Roboto Mono, Menlo, Consolas, "DejaVu Sans Mono", monospace`). No web font is loaded, so there is no font licence to carry. Weights are limited to 400, 600 and 700 because those exist on the common system fonts. Layout width 1240 px, gutters 20 px on phones, 28 px from 640 px and 40 px from 980 px, sections 88 px from 980 px and 56 px below.

Status is never shown by colour alone: every chip carries a word, backed by an outline style and a marker. Planned: dashed outline, hollow ring. In progress: solid outline, filled dot. Not started: solid outline, dashed ring. Not measured: solid outline, short bar. Optional reuses the planned shape; it never appears in the same list and the word carries the meaning.

## 4. Brand assets: provenance and licences

The concept PNGs in [assets/](assets/) are generated references (see [PROMPTS.md](PROMPTS.md)). They were not traced, edited or deleted.

| Asset | Origin |
|---|---|
| `brand/join-icon*.svg` | Original vector geometry authored 23 November 2026. The preferred "01 / Join" concept was measured by sampling the board pixel by pixel (strand 42 px, gap 21 px, 45° diagonals, chamfer legs 55 px outer and about 25 px inner) and regularised on a unit grid (strand 10, gap 5) by [tools/make-brand.mjs](tools/make-brand.mjs) |
| `brand/lockup*.svg` | The same mark plus an original monoline lowercase wordmark drawn from lines, circular arcs and one filled parallelogram. No font outlines are used |
| `brand/favicon.svg` | A small-size optical variant: the three entering strands do not survive 16 px, so only the chamfered bowl, its middle bar and the diagonal stem remain. Every coordinate is even on a 32-unit grid so edges land on pixel boundaries at 16 and 32 px |
| `brand/app-icon.svg` | The full mark on a tile, for 48 px and above |
| `brand/favicon.ico` | 16, 32 and 48 px PNG frames rasterised from `favicon.svg` by headless Chromium |
| `brand/apple-touch-icon.png` | 180 px, full-bleed square (iOS applies its own mask) rasterised from `app-icon.svg` |

Regenerating reproduces every committed file byte for byte (checked with `cmp`). The inline brand markup in `index.html` and `404.html` contains the same path strings as the SVG files (all 14 and 3 checked).

Forms delivered: monochrome, reversed (light on dark), two-tone (teal strand), icon-only and horizontal lockup. Legibility at 16, 24, 32 and 128 px on ivory and ink was checked by rendering and looking at the true rasters and pixel zooms: `after-icons-16-24-32-128-light-dark.png` in the screenshot folder.

Licences: all of these are original works made for this project; they contain no third-party artwork or font outlines. The public site repository has no licence file; choosing one is an owner decision. The brand guardrails hold: no shields, checkmarks, locks, sparkles or "verified AI" imagery. A distinctiveness or trademark search was not performed.

Test fonts (never shipped): screenshots use Selawik 1.01 (MIT) as a metric-compatible stand-in for Segoe UI, JetBrains Mono 2.304 (OFL) for monospace, DejaVu Sans 2.37 as a wide stress font, and Inter 4.0 (OFL) as an SF-like alternative. The browser image's own default is WenQuanYi Zen Hei, which would have misrepresented the design.

## 5. Behaviour

- Theme: CSS `prefers-color-scheme` does the work. The one inline script adds a toggle whose label names the target state ("Dark theme", accessible name "Switch to dark theme"). An explicit choice lasts for the page view only. Nothing is written to storage. Without JavaScript the page is complete and the toggle is hidden.
- Disclosure: the four non-displayed captures sit in a native `<details>`; it toggles by click, Enter and Space.
- Code block: focusable region with a name, so keyboard users can scroll it.
- Reduced motion: smooth scrolling is enabled only under `prefers-reduced-motion: no-preference`; there are no animations or transitions.
- Print: always light, navigation and controls removed. A real bug was found here: the dark rule's selector outranked the print rule, so a dark-mode visitor would have printed dark colours.
- Forced colours: links gain an underline; the brand mark keeps its colours.

## 6. Checks that were run, with results

Tooling: image `mcr.microsoft.com/playwright@sha256:1507cbfc550a63a6b7396791ca4f00df17beb56342d2447b3f106c306f525579` (tag `v1.65.1-noble`; Chromium 159.0.8154.6, Firefox 159.0, WebKit 27.2), `playwright-core` 1.65.1, `axe-core` 4.14.1, Node 26.13.1. How to reproduce: [tools/README.md](tools/README.md).

All numbers below are for the published revision `bc14aee`, whose live files were matched to the staged tree by SHA-256.

**Accessibility (automated).** axe-core 4.14.1 ran 50 distinct rules on the homepage (31 tagged WCAG 2.x A/AA, 19 best-practice).

| Run set | WCAG A/AA runs | Violations | Incomplete | Best-practice runs | Violations | Incomplete |
|---|---|---|---|---|---|---|
| Staged site: `file://`, root HTTP, sub-path HTTP; 2 pages, 3 viewports, 2 themes | 24 | 0 | 0 | 24 | 0 | 0 |
| Live URL, 2 pages, 3 viewports, 2 themes | 12 | 0 | 0 | 12 | 0 | 0 |
| Wide-font stress (DejaVu Sans), 2 pages | 12 | 0 | 0 | 12 | 0 | 0 |
| Firefox 159.0, desktop and mobile, 2 themes | 8 | 0 | 0 | 8 | 0 | 0 |
| WebKit 27.2, desktop and mobile, 2 themes | 8 | 0 | 0 | 8 | 0 | 0 |

"Runs" count page × viewport × theme combinations. Rules with nothing to test on this page do not appear in axe's results. The only rule tagged for a WCAG 2.2 success criterion that ran was `target-size` (2.5.8); the other 2.2 AA criteria have no dedicated axe rule at this version and are in the manual list below.

**Accessibility (measured by the runner, not by axe).**

| Check | Result |
|---|---|
| Contrast in both themes, hover and keyboard-focus states | every token pair at least 4.60:1 (lowest: copper on panel, light); 24 hover stops and 204 keyboard focus stops, 0 contrast violations |
| Focus indicator | 18 distinct focusable elements, all with a solid 3 px outline when tabbed to, none off-screen or covered |
| Target size | 10 distinct controls on the homepage; smallest 44 px, buttons 48 px; no inline text links |
| Reflow | 9 widths from 320 to 1920 px: no horizontal scroll, no element past the viewport, no clipped text, no overlapping text |
| Text spacing (WCAG 1.4.12 bookmarklet rules) | no loss of content at desktop, tablet, mobile |
| Reduced motion | `scroll-behavior: auto`, zero moving elements, both pages |
| Without JavaScript, system dark | dark page rendered, heading and navigation present, toggle hidden |
| Toggle and disclosure | toggle flips both ways; disclosure toggles by click, Enter and Space |
| Forced colours (Chromium emulation) | usable; all links underlined; no horizontal scroll |
| Print (Chromium emulation, forced dark system) | white background, navigation, skip link, toggle and buttons removed |
| Landmarks and headings | header, nav, main, one named region, footer; one `h1`, then `h2` and `h3` in order |

**Markup.** W3C Nu Html Checker: 0 errors on both pages. Seven informational notes say `role="list"` is unnecessary; it is kept on purpose because it preserves list semantics in Safari when `list-style` is removed.

**Wording.** The two required strings ("not a running Plyrune engine", "Not measured yet") are present. A lint for the handover's forbidden phrases ("verified agent", "correct edits", "safe at any time", "all callers", "prevents stale edits"), repository URLs, install commands, percentages and trial counts found nothing. The words proof, prove, correct, install, fast and repository each occur only in a negation, a plan statement or the approved heading.

**Performance (measured bytes, not estimated).**

| File | Uncompressed | gzip | brotli |
|---|---|---|---|
| `index.html` | 27,917 B | 7,954 B | 7,494 B |
| `styles.css` | 19,537 B | 4,874 B | 4,622 B |
| Homepage HTML + CSS + JS | 47,454 B | 12,828 B | 12,116 B |
| Whole publishable set (8 files) | 55,667 B | | |

Live transfer observed for the homepage: 8,031 B (document) + 4,945 B (stylesheet) = 12,976 B over the wire. Requests at load: exactly two for the homepage and one for the 404 page, all same-origin. No web font, no image beyond the icons, no script file.

**Hosting modes.** Rendered and checked from `file://`, from a local server at the root, from a local server under `/plyrune/`, and from the live root URL. Every link and asset reference is relative (five file references and eight in-page anchors); there are no absolute or root-absolute URLs, no `CNAME`, and `.nojekyll` is present.

**Before, for comparison (same tool, same fonts).** Prototype at `c375f59`: 18 runs; axe 0 violations, 6 incomplete (`aria-label` on a `<pre>`); three navigation targets 21.7 px tall; horizontal scroll at 320 px; overflow under text spacing at 390 px; light page for a dark-system visitor; no favicon.

## 7. How to publish

**Plan rule.** On GitHub Free, GitHub Pages is included "in public repositories"; publishing from a private repository needs a paid plan (GitHub Team for an organization, per GitHub's plans documentation). So a site for this organization must live in a public repository. Two shapes are possible:

- (a) a public `plyrune-dev/plyrune-dev.github.io` repository, published at the root URL `https://plyrune-dev.github.io/`;
- (b) make `plyrune` public and publish project Pages at `https://plyrune-dev.github.io/plyrune/`.

This is an owner decision. On the owner's authorisation, relayed on 23 November 2026, shape (a) was taken: `plyrune-dev/plyrune-dev.github.io` was created public and the implementation repository `plyrune-dev/plyrune` was left private and untouched. The site was also verified to work under a sub-path, so (b) remains available later.

**Steps used.**

1. `website/tools/stage.sh DEST` copies the allowlisted files into an empty directory.
2. Check the staged directory (see [tools/README.md](tools/README.md)).
3. Replace the site repository's working tree with the staged files, commit, push `main`. Pages builds from `main` at the repository root; HTTPS is enforced; builds take under a minute.
4. Verify: the Pages build status for the pushed commit, the live `index.html` SHA-256 against the staged file, each asset's status and content type, and the runner against the live URL.

Published commits: `bba2cf3` (prototype preview, 23 Nov 22:49 UTC), `8cfba6e` (production site, 24 Nov 00:26 UTC), `171abf3` (footer note, 24 Nov 00:48 UTC), `bc14aee` (responsive gutters, 24 Nov 01:37 UTC). Each was verified live after its build.

Limits worth knowing: GitHub applies a soft limit of 10 builds per hour and 100 GB bandwidth per month; the CDN caches for 10 minutes (`cache-control: max-age=600` was observed); GitHub Pages logs visitor IP addresses for security, which the footer now states.

## 8. Website-specific critique

Gaps in the proposal that this work closed:

1. Deployable and non-deployable material were mixed in one directory with no notion of what ships. Now: an explicit allowlist; 8 files published; concept art and notes stay private.
2. No hosting target had been exercised. Now: `.nojekyll`, a 404 page, relative paths, verified at root, sub-path and `file://`.
3. "Test at 16, 24 and 32 px", "qualify favicon legibility", "wire real vectors" were listed as to-dos with no assets. Now: original vectors with a pixel-snapped small variant, inspected as rasters.
4. The page ignored the system colour scheme and offered a control that did nothing without script.
5. Assurance copy listed claims without the statements a reader could hold the project to. Now every claim is quoted with the control meant to break it.
6. Measurements, roadmap and evaluation had no dated status, so they could silently go stale.

Risks that remain:

- Dated content ("as of 23 November 2026", "M0 in progress") will be wrong soon. It needs a refresh rule, not a rewrite.
- The claim list is a hand copy of [../docs/ASSURANCE.md](../docs/ASSURANCE.md) and will drift when that file changes.
- Screenshots prove layout under stand-in fonts, not under the fonts visitors actually have.
- The 404 page's home link is relative, so it resolves correctly for missing paths directly under the site base but may not for deeper missing paths.

Owner decisions:

1. Repository link: the implementation repository is private, so the footer says so and links nothing. Change it when the repository becomes public.
2. Open Graph image and canonical URL: omitted because both need an absolute URL. The root URL is now fixed, so adding `og:image` is a small follow-up if link previews matter.
3. Licence for the public site repository (none today).
4. Public repository settings: Issues and Wikis are on by default, and every push to `main` goes live within about a minute. A pull-request flow and disabling Issues are worth considering.
5. Whether to persist the theme choice. It is deliberately not stored.
6. Distinctiveness and trademark check for the mark before any wider announcement.
7. A Content-Security-Policy meta tag as extra hardening (none is set; GitHub sends none either).
8. Whether the historical pilot of the predecessor engine should ever be summarised publicly. It is not mentioned.

## 9. Not verified

- Real devices and browsers: Safari on macOS and iOS, Chrome on Android, Edge. Firefox and WebKit were exercised only through Playwright on Linux.
- Real system fonts. Screenshots use stand-ins.
- Screen readers, voice control, switch access, and keyboard-only use by a person.
- Browser zoom at 200 % and 400 %. Emulated by viewport widths of 640 px and 320 px only.
- Windows High Contrast Mode and printed paper. Both were emulated in Chromium.
- Favicons in real browser tabs, bookmark bars and iOS home screens. The rasters were inspected directly.
- Sub-path hosting on GitHub Pages. Only root hosting is live; the sub-path was simulated with a local server.
- Link previews on social platforms (no `og:image` exists).
- Lighthouse, PageSpeed or slow-network behaviour. Only byte counts and request counts were measured.
- Firefox did not complete the optional stages (no-JavaScript, toggle, disclosure, reduced motion): the browser process closed when the harness opened a further context, and the run was stopped at a 300 s cap after its 8 page runs. WebKit and Firefox logged messages that came from the harness itself (its own stylesheet fetch over `file://`, and Playwright's layout probe); each page's own load was clean.

## 10. Copy boundaries (unchanged from the handover)

- "Complete selection" always means complete for a declared predicate and scope.
- A checked candidate names its requirement and checker profile.
- Avoid "all callers" for literal or regex matching.
- Describe origin and freshness precisely; a digest is not a permission token.
- Keep historical engine results distinct from new Plyrune composition and performance results.
- Keep private playbook and archive material out of public assets until separately approved.

The three identity directions (Join, Selvedge, Notch) and the reasons for preferring Join are recorded in [PROMPTS.md](PROMPTS.md) and [../HANDOVER.md](../HANDOVER.md). Join remains the direction; Selvedge and Notch were not developed further.
