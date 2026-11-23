# Brand and website design handoff

Recommendation: **Join** mark + **Ivory / evidence** homepage. Keep the dark workbench as an alternative or documentation theme.

The three generated PNG boards in `assets/` are concept references. They are not production vector logos. The offline HTML uses a text wordmark so it remains self-contained; its semantic copy is authoritative when a generated mockup differs. No site was deployed.

## Identity directions

| Direction | Idea | Assessment |
|---|---|---|
| 01 · Join | Three strands converge into one P-shaped path | Preferred: expresses unification and has a strong small silhouette |
| 02 · Selvedge | Interlocking perpendicular textile bands | More architectural; attractive but busier at favicon size |
| 03 · Notch | A carved P with deliberate cuts | Most compact; less directly communicates shared flow |

Use lowercase `plyrune`, pronounced PLY-roon. Avoid shields, checkmarks, locks or “verified AI” symbolism. The brand should communicate continuity and precision without implying that arbitrary agent output is proved correct.

Before production: redraw the selected concept as an approved original vector master; provide monochrome, reversed, horizontal and icon-only lockups. Test at 16, 24 and 32 px and on light/dark backgrounds. Make optical small-size corrections explicitly rather than shrinking a detailed mark. Retain concept provenance and check distinctiveness before launch.

## Design system

| Token | Value | Use |
|---|---|---|
| Paper | `#F5F2EA` | Warm public background |
| Ink | `#172923` | Main text, rules, strong panels |
| Teal | `#236A57` | Primary action and source/path accents |
| Copper | `#A84F32` | Preview and qualification labels |
| Dark paper | `#111F1B` | Workbench background |
| Dark accent | `#96D0B8` | Legible interaction color on dark |

Use a clean sans-serif with a restrained human character and monospace for identities, code and data. The prototype uses installed/system fonts; it makes no external font request. Select and license production fonts separately. Use tabular numerals for measurements and thin rules rather than a dashboard full of cards.

Desktop content width approximately 1240 px, generous 64–76 px section spacing, 40 px gutters. On mobile use 20 px gutters, stack the flow and evidence panels, keep code horizontally scrollable, and maintain visible focus and 44 px actions. Respect reduced-motion preference. Do not encode status by color alone.

## Public page order

1. Product promise: “Source to change, without losing the thread.” Preview badge until implemented.
2. One-interface flow: query, optional inspect, prepare, check. Candidate-only boundary near the diagram.
3. Retained-selection example: two displayed captures with the remaining four expandable; static fixture clearly labeled.
4. Assurance: exact property statements and their evidence/revision links once available.
5. Measurements: task outcomes, baseline, denominators, uncertainty and raw data. Current state is “Not measured yet.”
6. Roadmap and evaluation method.

Launch navigation can add Docs, Examples, Assurance, Measurements and GitHub after those destinations exist. Do not fabricate install commands, repository URLs, customer logos, stars, trial counts or speedup badges.

## Copy boundaries

- “Complete selection” always means complete for a declared predicate and scope.
- “Checked candidate” names its requirement and checker profile.
- Avoid “all callers” for literal or regex matching.
- Describe origin/freshness precisely; a digest is not a permission token.
- Keep historical engine results distinct from new Plyrune composition and performance results.
- Keep private playbook/archive material out of public assets until separately approved for publication.

## Assets and prototype

- `assets/identity-studies.png`: Join, Selvedge and Notch logo concepts.
- `assets/join-transparent.png`: transparent PNG of the preferred Join icon, suitable for prototype use; still a raster concept, not a production vector master.
- `assets/website-ivory.png`: recommended desktop visual direction.
- `assets/website-workbench.png`: alternative dark desktop visual direction.
- `index.html`: offline responsive reference with theme toggle, anchor navigation and expandable fixture inspection. It does not call an agent engine.
- `measurements.json`: truthful empty state for future generated website data.
- `PROMPTS.md`: exact concept-generation prompts and provenance.

The prototype is a design artifact, not a production website. Before deployment, perform browser/device/accessibility checks, wire real documentation and admitted result data, and replace the text-only brand treatment with the approved vector asset. Keep public claims synchronized with source evidence.
