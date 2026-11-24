# Website tools

Development-only. Nothing in this directory is published. Everything runs inside the pinned headless-browser image, never with tools installed on the host.

## Publish allowlist and staging

- `publish-allowlist.txt` is the only list of files that may be published. `stage.sh DEST` copies just those paths into an empty directory and prints their sizes. Publishing is then a manual copy of that directory into the site repository.
- `tools/validate_handover.py` at the repository root is unchanged and must pass in the worktree at every commit. Run it without piping its output, so a failure stops you: `python3 tools/validate_handover.py`.

## Pinned tooling

| Item | Value |
|---|---|
| Image | `mcr.microsoft.com/playwright@sha256:1507cbfc550a63a6b7396791ca4f00df17beb56342d2447b3f106c306f525579` (tag `v1.65.1-noble`, Ubuntu 24.04, Node 26.13.1, Chromium 159.0.8154.6, Firefox 159.0, WebKit 27.2) |
| npm | `playwright-core` 1.65.1, `axe-core` 4.14.1, pinned in `package.json` and `package-lock.json` (`npm ci` in a scratch directory) |
| Resources | `--cpus 2 --memory 2g`, run as the invoking user, site and tools mounted read-only |

Mounts used by the scripts: site root at `/site`, this directory at `/tools`, the `node_modules` directory at `/tooling`, output at `/out`. Run a tool by copying it into the container next to a `node_modules` symlink; `AXE` must point at `axe.min.js`.

## Tools

| Tool | Purpose |
|---|---|
| `sitecheck.mjs` | Loads the site in headless browsers and reports layout, axe-core, focus and hover contrast, target size, reduced motion, text spacing, forced colours, print, references, wording and byte weights; takes screenshots |
| `summarize.mjs` | Prints the key facts of a report as plain lines |
| `make-brand.mjs` | Writes the brand SVGs from their parameters |
| `render-icons.mjs` | Rasterises `favicon.ico` and `apple-touch-icon.png` |
| `iconsheet.mjs` | Renders the legibility sheet (true sizes and pixel zooms, both backgrounds) |
| `htmlcheck.mjs` | Sends HTML to the W3C Nu Html Checker (needs network) |
| `crop.mjs`, `sheet.mjs` | Inspect screenshots: crop and enlarge, or lay a tall page out in columns |

Typical invocations (arguments after the tool name):

```
sitecheck.mjs --root /site --pages index.html,404.html --modes file,root,subpath --out /out/after --prefix after --shots --full --pairs ink/paper,muted/paper
sitecheck.mjs --root /site --url https://plyrune-dev.github.io/ --pages index.html,404.html --shotMode live --out /out/live --prefix live --shots
sitecheck.mjs --root /site --font wide --out /out/wide --prefix wide --full
sitecheck.mjs --root /site --browsers firefox --out /out/ff --prefix ff
summarize.mjs /out/after/after-report.json
```

`--font` selects the stand-in for the system font stack: `segoe` (Selawik, default), `inter`, `wide` (DejaVu Sans) or `container` (the image's own default, not representative). `--toggle SELECTOR` clicks a theme button for pages that do not follow `prefers-color-scheme`.

## Reading a report

- `load*` lists are the page's own: console messages, failed requests and errors up to the moment the page finished loading, before any tooling was injected.
- `afterTooling*` lists are recorded after axe-core was injected and screenshots were taken. Under `file://` the harness itself fetches the stylesheet and Chromium logs a CORS message for it; that is tooling noise, not a page request.
- An empty list means the probe found nothing. It does not mean a person checked it. Items that need a person are listed under "Not verified" in `DESIGN.md`.
- A browser that hangs is reported, not waited for: launching is capped at 90 s, and the summary is written even when a stage fails.

## Safety notes

- Run containers as the invoking user where possible. A container running as root must never have a host directory mounted read-write unless every path it writes is validated: an empty variable in a `cp` source once turned into a copy of the whole container root filesystem into the mount.
- Prefer streaming results out (tar on stdout) over writing into bind mounts.
- Firefox in this image closes when the harness opens a context after its page runs; wrap it in `timeout`.
