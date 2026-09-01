# telividb — brand and voice

The design and the copy already follow rules. This writes them down, because
until now they lived in one person's head and in scattered CSS comments, and a
contributor could not reproduce them from the code alone.

Everything here is **descriptive**: it was derived from what the app already
says and shows, not invented for it. Where a rule has a `CLAUDE.md` invariant
behind it, that is cited — the voice is downstream of the engineering
guarantees, never the other way round.

> **Do not run `sync-brand-to-tokens.cjs` against this file.** It generates
> `assets/design-tokens.json` and `.css`, and the tokens already live in
> `app/src/app.css`. A generated second copy that nothing imports is exactly the
> drift this project spends effort avoiding — the Storybook preview loads the
> app's real stylesheet for the same reason. This document *describes* the
> palette; `app.css` *is* it.

---

## Brand Personality

telividb is a database with a window on it, for people who will read what it
says. It behaves like good instrumentation: it tells you what is true, including
when the answer is "this does not work yet", and it never claims a guarantee it
cannot keep.

## Core Attributes

- **Precise** — names the specific thing: the RPC, the crate, the file
- **Honest** — states limits on the screen, not in a footnote
- **Explanatory** — says what happens next, not only what the state is
- **Unhurried** — no urgency, no persuasion, no exclamation marks

---

## Voice rules

**1. Name what is missing.** Never "coming soon". The app says
*"Running on sample data — Graph.ListEdges is not served yet"* and
*"Needs the policy engine and a vault to unlock"*. A reader can act on a named
gap and cannot act on a promise.

**2. Say the consequence, not the state.** *"A vector made from a secret leaks
it"* rather than "Redaction: off". The state is visible; the consequence is the
part they cannot see.

**3. Never let language outrun the guarantee.** This is
[rule 25](../CLAUDE.md) in prose. *"Readable only by its owner, enforced by a
visibility predicate. Access control, not cryptography"* — because calling that
a vault would invite the assumption that it survives a compromised server. Only
a key-wrapped collection is a vault; only a client-held key is sealed.

**4. Refuse rather than warn** where a guarantee is at stake. A vault whose
passages were sent to a third party after someone clicked through a dialog was
never a vault.

**5. Distinguish absence from concealment.** *"No results"* and *"no results
you can currently see"* are different answers, and a locked space says which
one it is (rule 27).

**6. Explain the permanent thing at the moment it becomes permanent.**
*"The id derived from it is permanent"* appears while the field is still empty —
the only moment saying so helps.

**7. Errors are sentences, not codes.** They say what was refused and what to do
instead: *"choose a local provider, or ask in a private space"*.

---

## Forbidden Phrases

- **Coming soon** — name the service or crate that has to land
- **Secure**, **bank-grade**, **military-grade** — say what the cryptography
  actually does, or say access control
- **Enterprise-grade**, **blazing fast**, **seamless**, **effortless**
- **Something went wrong** — say what went wrong
- **Are you sure?** — say what will happen if they continue
- **Vault** for anything not key-wrapped (rule 25)
- **Simply**, **just**, **easy** — they blame the reader when it isn't
- Exclamation marks in product copy

---

## Style Keywords

Flat, gridded, high-contrast, monospaced-for-figures, square-cornered,
signage-like, instrument-panel, unhurried.

## Visual Mood Descriptors

A transit information board rather than a dashboard: one grid, few sizes, and
alignment doing the work decoration usually does. The reference is the NYCTA
Graphics Standards Manual — the whole system carries a `--radius: 0`, and the
sharp corner is the single most identifying decision in it.

## Visual Don'ts

- Rounded corners, shadows, gradients, glows
- Colour as the only carrier of meaning — always a word or a shape too
- A size not on the 4px module
- Emoji, or a text glyph standing in for an icon without a label
- Motion that decorates rather than sequences; two systems animating one
  property

---

## Colour

`app/src/app.css` is authoritative. This is a description of what each role
asserts — the reason a colour is used, which is the part a hex value cannot say.

### Primary Colors

- **Ground** `--color-ground` — the window itself. Dark is the default, not a
  preference: this is read for hours beside a terminal.
- **Ink** `--color-ink` — anything the reader must be able to read.

### Semantic

- **Green** `--color-green` — ready, resident, *runs on this machine*. The only
  colour that carries a safety claim, and it is used only where that claim is
  true.
- **Amber** `--color-amber` — attention, and the honesty vocabulary:
  `sample data`, `not enforced`, `not built`, `locked`. It is darkened for the
  light theme, because the dark value measures 1.52:1 on a light ground — which
  would erase exactly the marks that exist to prevent a misunderstanding. Both
  values, and the ratios, are in `app.css` beside the declaration.
- **Red** `--color-red` — refused or failed. Never used for emphasis.
- **Blue** `--color-blue` — focus, citation, and the first chart series.

### Neutral

`--color-surface`, `--color-sunken`, `--color-rule`, `--color-rule-strong`,
`--color-ink-dim`, `--color-ink-faint` — elevation and hierarchy. Every one of
these carrying text clears 4.5:1 on both grounds; that is checked, not assumed.

### Prohibited

- Amber or red as a chart series — both failed the lightness band on the dark
  ground, so they stay status-only
- Any accent below 4.5:1 on either theme's ground

---

## Typography

Geist and Geist Mono, vendored rather than linked — a font CDN fails with no
network, which a desktop app must survive, and reports every launch to a third
party.

Monospace is not decoration: it marks **figures that must line up or be
compared** — scores, dimensions, byte counts, resource names, ids. Proportional
type is for prose.

---

## Base Prompt Template

When writing UI copy for telividb:

```
Write in the voice of telividb: precise, honest, explanatory, unhurried.
Name the specific missing thing rather than promising it. State the
consequence rather than the state. Never claim a guarantee stronger than
what is implemented — access control is not cryptography. Distinguish
"nothing here" from "nothing you can currently see". No exclamation marks,
no "simply", no "coming soon", no "something went wrong".
```

---

## Checking this document against the app

It describes real values, so it can go stale. Two checks:

```bash
# The palette this document describes is the one app.css defines.
grep -E "^\s+--color-(ground|ink|green|amber|red|blue):" app/src/app.css

# No forbidden phrase reached the UI.
grep -rniE "coming soon|bank-grade|something went wrong|blazing fast" \
  app/src/lib --include='*.svelte'
```

The second is worth running before a release. The first is worth running when
this file is edited.
