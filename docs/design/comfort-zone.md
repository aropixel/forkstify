# Comfort zone

Working note. **Decided** = recorded in `docs/decisions/`; **direction** =
proposed, uncontradicted, not yet recorded; **to settle** = open question.

## Decided

- A dial from **0 to 5**, set **at launch**.
- It measures **familiarity** ([0001](../decisions/0001-comfort-is-familiarity.md)):
  **5 = cocoon, 0 = exploration** — *flipped on 2026-09-06*. The note said
  the opposite; on the first real use, Joel: "if I want the cocoon, I should
  set comfort to 5 — comfort is what you know well". He is right, and the
  repository was the odd one out:
  [0012](../decisions/0012-track-rotation.md) §4 writes "high comfort: a
  tight draw on the tops", which now reads literally. A design note yields
  to use.
- It is what **chooses on its own** at a fork point when the user does not
  actively choose. The application never blocks.

## Wired on 2026-09-05

`engine::Comfort` (0 to 5), read from `~/.config/forkstify/config.toml`
(`[journey] comfort`) and adjustable while listening with `:comfort <n>`. It
acts on two levers, the ones `avancement.md` already called "constants to be
driven by comfort":

- **The floor of the adventurous branch** drops as you open up: cosine ≥
  0.80 at the cocoon, ≥ 0.60 at exploration. **Comfort 2 reproduces exactly
  the fixed setting from before** (0.72 / 0.80) — the old agreement becomes
  the middle of the dial, not a lost value.
- **Familiarity tilts the draw of branch heads** (0001: comfort *is*
  familiarity). At the cocoon, a familiar artist goes ahead of an unknown
  one; wide open, the reverse. The factor is bounded to [0.25, 2.0]: **we
  discourage, we never forbid** — the application does not decide in the
  ear's place.

**The polarity trap, never to be reopened without reading this.**
[0012](../decisions/0012-track-rotation.md) §4 writes "high comfort: a tight
draw on the tops; low comfort: the long tail weighs more". That "high
comfort" means the **feeling** of comfort — the cocoon — that is, the value
**0** on that scale, not 5. Read literally with "5 = exploration", the dial
inverts entirely. A test pins it down
(`le_cocon_penche_vers_le_connu_et_l_exploration_vers_l_inconnu`).

A detail of shape: with six integer values, **there is no exact middle**. 2
still leans a little towards the known, 3 already a little towards the
unknown; the tipping point falls between the two.

## Directions

- **The dial structures the range on offer**, not just the default choice:
  at every fork point, the branches are spread along the axis — one more
  reassuring than the setting, one at its level, one more adventurous. The
  user builds a stable mental model ("left reassures me, right takes me
  out") and chooses without reading.
- **The dial can be adjusted mid-journey.** The value at launch is only a
  starting point.
- **Technically, comfort is a distance** between a candidate artist and the
  center of gravity of what the user knows, in the vector space described in
  [branch-engine.md](branch-engine.md).

## To settle

- ~~How the application knows what the user **knows**~~ — **settled de facto
  by [0014](../decisions/0014-shape-of-the-learned.md)**: it is `learned/`.
  Our decayed plays first (saturating — the tenth play says much less than
  the first), and failing that `classement.json`, brought onto the same 0–1
  scale by its own maximum, since one is a count and the other a composite
  score. Left out of the computation: the cards changed in the fork, which
  could weigh in one day.
- How long the application waits at a fork point before choosing on its own:
  until the end of the segment, a fixed delay, or no wait at all (it carries
  on and the user can deviate at any time)?
- Is comfort a dial set by the user, an indicator shown by the application,
  or both?

## Its place on screen (2026-09-14)

Joel: the gauge must be **always in the same place on both screens, at the
top right, and always with the look it has while editing**. Done. A single
rendering (`comfort_spans` in `tui.rs`) serves home and listening: the
blocks keep their color, the label stays lit permanently (the black on cyan
that only served in `cc` mode), and `cc` adds "↑↓" to say the gauge is live.
On home, the gauge takes the place the status indicators had.

**Home's status is only shown when degraded** (Joel's question: "are these
pieces of information really useful?"). librespot, the web API and the sync
only show up when one of them is off (in red, on the second line ahead of
the census): all green, we show nothing and the space goes to comfort. That
is where they are useful — a lost auth, an unreachable API, a failed sync.

**Comfort is kept from one launch to the next** (Joel, 2026-09-14). A state
file `~/.local/state/forkstify/comfort` holds the last chosen value; it is
written on every change (`c<n>`, `:comfort`, a confirmed `cc`, on home as
while listening) and read back at startup. The `comfort` in `config.toml` is
now only the seed of the very first launch: as soon as you adjust it, the
state wins. That answers the third open question above — comfort is indeed a
dial set by the user, and it persists.
