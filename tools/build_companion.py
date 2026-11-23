"""Build the seven-page visual companion; full implementation detail is Markdown."""
from pathlib import Path
from reportlab.pdfgen import canvas
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, PageBreak, Table, TableStyle, Image, KeepTogether
from reportlab.lib.styles import ParagraphStyle
from reportlab.lib import colors
from reportlab.lib.pagesizes import A4
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/'Plyrune-Handover.pdf'
FONTS = Path('/usr/share/fonts/truetype/open-sans')
pdfmetrics.registerFont(TTFont('OpenSans', str(FONTS/'OpenSans-Regular.ttf')))
pdfmetrics.registerFont(TTFont('OpenSansBold', str(FONTS/'OpenSans-Bold.ttf')))
pdfmetrics.registerFontFamily('OpenSans', normal='OpenSans', bold='OpenSansBold')
INK = colors.HexColor('#172923')
TEAL = colors.HexColor('#236A57')
COPPER = colors.HexColor('#A84F32')
LINE = colors.HexColor('#CFD5CB')
PAPER = colors.HexColor('#F5F2EA')
W, H = A4
WIDTH = W-96
S = {
    'kicker': ParagraphStyle('kicker',fontName='OpenSansBold',fontSize=8,leading=12,textColor=COPPER,spaceAfter=10),
    'title': ParagraphStyle('title',fontName='OpenSansBold',fontSize=32,leading=37,textColor=INK,spaceAfter=16),
    'h1': ParagraphStyle('h1',fontName='OpenSansBold',fontSize=23,leading=28,textColor=INK,spaceAfter=15),
    'h2': ParagraphStyle('h2',fontName='OpenSansBold',fontSize=12,leading=17,textColor=TEAL,spaceBefore=13,spaceAfter=7),
    'body': ParagraphStyle('body',fontName='OpenSans',fontSize=10,leading=15,textColor=INK,spaceAfter=9),
    'small': ParagraphStyle('small',fontName='OpenSans',fontSize=8,leading=12,textColor=INK,spaceAfter=7),
    'cell': ParagraphStyle('cell',fontName='OpenSans',fontSize=8.2,leading=12,textColor=INK),
    'head': ParagraphStyle('head',fontName='OpenSansBold',fontSize=8.2,leading=12,textColor=colors.white),
}
story=[]
def p(s, style='body'):
    story.append(Paragraph(s,S[style]))
def title(n,s):
    p(f'PLYRUNE / {n:02d} / REVIEWED DESIGN', 'kicker')
    p(s,'h1')
def table(rows,widths):
    data=[[Paragraph(str(v),S['head'] if i==0 else S['cell']) for v in row] for i,row in enumerate(rows)]
    t=Table(data,colWidths=widths,repeatRows=1,hAlign='LEFT')
    t.setStyle(TableStyle([('BACKGROUND',(0,0),(-1,0),TEAL),('VALIGN',(0,0),(-1,-1),'TOP'),('LEFTPADDING',(0,0),(-1,-1),9),('RIGHTPADDING',(0,0),(-1,-1),9),('TOPPADDING',(0,0),(-1,-1),9),('BOTTOMPADDING',(0,0),(-1,-1),9),('LINEBELOW',(0,1),(-1,-1),.5,LINE)]))
    story.append(t)
    story.append(Spacer(1,10))
def art(name):
    image=Image(str(ROOT/'website/assets'/name),width=WIDTH,height=WIDTH*2/3)
    story.append(image)
    story.append(Spacer(1,14))
def page(): story.append(PageBreak())

p('IMPLEMENTATION HANDOVER / 23 NOVEMBER 2026','kicker')
p('plyrune','title')
p('One interface from source<br/>to checked change.','h1')
p('Unify Tuft, Weft and Splicery around immutable captures, explicit scope and reports bound to exact candidates. Preserve the existing proved kernels; qualify the composition between them.')
art('identity-studies.png')
p('Recommended identity: <b>01 / Join</b>. Three strands become one recognizable path. The ivory homepage is the preferred public direction. These are generated concepts; production vectors are still required.','small')
p('<b>Status:</b> independently reviewed design, implementation not started. No Plyrune benchmark, new composition proof, domain registration or deployment was performed.','small')
page()

title(2,'The shared artifact contract')
p('The agent queries once and carries the same captures into inspection, candidate preparation and checks. A bounded display never silently narrows the retained selection.')
table([
['Artifact','Meaning and binding'],
['Snapshot','Immutable manifest with an explicit acquisition profile. A committed Git tree excludes dirty/untracked files.'],
['Requirement','Accepted scope, predicate, matcher and allowed dispositions, admitted separately from candidate selection.'],
['Capture / Selection','Exact resource/snapshot ranges; complete membership with a scope-bound inventory witness, including no-match files.'],
['View','Projection and source map with explicit omissions. A partial display is not an exact whole-source object.'],
['Change','Base, operations, read dependencies, write targets and immutable candidate identity.'],
['Check / Applicability','A report binds candidate, requirement, premises and checker. Applicability compares its dependencies with a current context.'],
],[116,WIDTH-116])
p('First vertical slice','h2')
p('Replace every exact <b>legacy_endpoint</b> occurrence in the admitted config/ domain with <b>service_endpoint</b>. The example has six occurrences across three changed files and one no-match file. A fifth resource outside config/ remains untouched.')
p('Check scope, exact occurrence coverage, permitted replacement, absence of extra edits, untouched bytes and candidate/report identity. A query narrowed to config/server/ must fail the wider requirement even if every returned match was edited.')
p('<b>Keep these separate:</b> historical validity, present applicability and authority to publish. A read-set recheck followed by an unprotected write still has a race.','small')
page()

title(3,'Six milestones, with a stop gate')
table([
['Milestone','Deliverable','Acceptance evidence'],
['M0\nContracts','Pins, Tauten tiers, shared model, extraction spike and native baseline','Reproducible bootstrap; admitted fixtures; explicit proof/runtime bridges'],
['M1\nComposition','Immutable manifest to literal selection, exact candidate and coverage report','Selected Lean claims over shipped code; adversarial controls; no workspace writes'],
['M2\nInterface','One CLI/stdio service, retained handles, bounded views and thin SDK','Packaged consumer; owner/snapshot refusals; early paired cost pilot'],
['M3\nRevalidation','Conservative dependency checks, then worthwhile incremental reuse','New-file, negative-match, deletion, exclusion and anti-join controls'],
['M4\nEvaluation','Native/facade/shared/incremental comparison and honest site results','Independent grading; all trials; raw usage; uncertainty; negative results'],
['M5\nPublication','One separately qualified host apply protocol, if needed','Actual concurrency/linearization mapping; fault/race tests; recovery scope'],
],[77,195,WIDTH-272])
p('Build the small useful path first','h2')
p('Start with one binary/library and three Rust crates: kernel, core and CLI. The first change pins contracts and gates; the second completes the pure migration; the third makes it usable by a real harness.')
p('Run the early pilot after M2. If success is at ceiling and the interface costs more, simplify or narrow the use case before adding M3 machinery. A smaller useful library is an acceptable outcome.')
p('Defer distributed runtimes, general-purpose DSL/VM, semantic search, automatic rebasing, arbitrary test-result caching and cross-repository publication.','small')
page()

title(4,'Engineering profile and Tauten')
p('The copy-ready AGENTS.md adapts the private engineering playbook at <b>cca8e068...</b>. It selects gates for the changed obligation; it does not demand every prover and model checker at once.')
table([
['Boundary','Required discipline'],
['Semantic kernel','Pure no_std decisions; nominal identities; checked construction; explicit assumptions; no ambient effects.'],
['Wire and imports','Untrusted DTOs validate into domain objects. Serialization, recovery and alternate entrypoints cannot bypass constructors.'],
['Proof correspondence','Use the production Rust translation. Preserve leaf proofs and classify canonicalization/acquisition/mapping bridges separately.'],
['Release behavior','Keep bounds, identity and correctness checks in release. Counters do not wrap; errors remain typed.'],
['Evidence','Separate source-inspected, tested, model-checked and proved. Missing tools and stale records are not passes.'],
],[116,WIDTH-116])
p('Tauten from M0','h2')
p('Pin CLI/runtime/templates deliberately. The inspected development revision is <b>bac9fdd1...</b>; documented Rust is 1.98.0. Run formatting, the Tauten profile and workspace tests. The Lean proof gate remains separate. Exact commands and pins are in docs/TAUTEN.md.')
p('Existing evidence, permits, bindings and declared effect rows can be reused. They do not infer query completeness, discover every dependency, or enforce filesystem concurrency for Plyrune.')
p('Two focused upstream findings','h2')
p('<b>Stamp identity:</b> the current carrier compares revision only, so distinct same-typed subjects at the same revision can pass the source-level check. Use explicit owner/snapshot/candidate bindings in Plyrune.')
p('<b>Capability table:</b> MAPPING.md still labels several shipped evidence/effect/proof capabilities as absent or planned. Both issue drafts were independently reviewed. Neither was posted: integration access was denied and browser passkey authentication failed.','small')
page()

title(5,'Direction A: ivory and evidence')
art('website-ivory.png')
p('The recommended public face','h2')
p('Warm ivory, deep green ink, restrained teal and copper. A clear product promise sits beside the typed flow. Assurance and performance occupy different panels with explicit states.')
p('The preview must say what is planned. No made-up benchmark percentages, proof-success badges, customer logos or install commands. A claim becomes public only with its exact source revision, scope, assumptions and evidence.')
p('The included offline HTML adds responsive stacking, keyboard focus, a light/dark toggle, anchor navigation and an expandable static capture fixture. It is a design prototype, not an agent engine.')
p('Production identity: approve and redraw the Join concept as vectors, then test 16/24/32 px sizes and reversed variants. The name checks were preliminary and nothing was reserved.','small')
page()

title(6,'Direction B: the workbench')
art('website-workbench.png')
p('A technical alternative','h2')
p('Dark forest background and monospaced artifact rows make provenance and scope prominent. This is suitable as a docs/workbench theme, while the ivory direction is warmer for the public homepage.')
p('Measurements to publish','h2')
p('Task success, time to correct result, model usage/cost, missed/unintended edits, false refusals and recovery. Separate cold setup from warm reuse. Keep failed trials in denominators and distinguish model roundtrips from internal operations.')
p('Compare competent native tools, a unified facade, shared artifacts and then incrementality. Every arm can script and batch. Use independent oracles and publish uncertainty and negative outcomes.')
p('<b>Current state: unmeasured.</b> Fewer returned bytes alone do not establish a better workflow. Historical Tuft/Splicery results are not new Plyrune results.','small')
page()

title(7,'Reviewer changes and next action')
p('The independent review materially tightened the design:','h2')
for x in [
    'Use immutable fixture/Git inputs first; do not call a changing multi-file scan a coherent snapshot.',
    'Bind completeness to scope, inventory and matcher semantics, including files with no matches.',
    'Admit requirements separately, so coverage cannot pass by narrowing the selected set.',
    'Keep historical validity separate from current applicability and bind every report to its exact subject.',
    'Defer live apply until a host can enforce the relevant concurrency contract.',
    'Begin with one process and test against competent native batching before growing the framework.'
]: p('- '+x)
p('Give the implementing agent this assignment','h2')
p('Read START-HERE.md and AGENTS.md. Complete M0. Then deliver the M1 literal-migration fixture through the production kernel and a mechanically checked report, with owner/snapshot/scope/partial-view negative controls. Do not implement live apply in that change.')
p('Package contents','h2')
p('HANDOVER.md, MILESTONES.md, AGENTS.md; shared contracts, assurance, Tauten adoption, evaluation and reviewer disposition; example inputs/expected candidate; three generated design boards; offline HTML; exact issue drafts and submission status; validation/build scripts. The Markdown documents carry the implementation detail omitted from this visual companion.','small')
p('Inspected source revisions','h2')
for label,url,pin in [
('Playbook (private)','https://github.com/tknawara/playbook/tree/cca8e068d0843af5612522049f08d8d645252f7c','cca8e068d084'),
('Tauten','https://github.com/tauten-dev/tauten/tree/bac9fdd1f4f2390f8c7fe2d95a6bc878e6149f42','bac9fdd1f4f2'),
('Splicery','https://github.com/writfold-dev/splicery/tree/632a90fd96e920fbac7ee5818f989739c1617062','632a90fd96e9'),
('Weft','https://github.com/writfold-dev/weft/tree/118c1bf3de82002e3070267a54f28eedaa1ad678','118c1bf3de82'),
]: p(f'<link href="{url}" color="#236A57">{label}</link> / {pin}', 'small')
p('No Rust/Lean gate or Plyrune benchmark was run in this authoring session. New composition claims remain planned. Tauten issue posting remains blocked; see issues/STATUS.md.','small')

def frame(c,doc):
    c.setFillColor(PAPER)
    c.rect(0,0,W,H,fill=1,stroke=0)
    c.setStrokeColor(LINE)
    c.line(48,42,W-48,42)
    c.setFont('OpenSans',7)
    c.setFillColor(INK)
    c.drawString(48,28,'PLYRUNE / IMPLEMENTATION HANDOVER / 23 NOV 2026')
    c.drawRightString(W-48,28,str(doc.page))

doc=SimpleDocTemplate(str(OUT),pagesize=A4,rightMargin=48,leftMargin=48,topMargin=46,bottomMargin=58,title='Plyrune - reviewed implementation handover',author='Prepared for Tarek',subject='Unified source, capture, candidate and check interface')
doc.build(story,onFirstPage=frame,onLaterPages=frame)
print(OUT)
