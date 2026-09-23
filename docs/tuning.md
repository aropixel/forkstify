# The engine's settings (`[tuning]`)

Every number the engine uses to decide what plays is a setting, in the
`[tuning]` section of `~/.config/forkstify/config.toml`
([0023](decisions/0023-engine-numbers-are-tunable.md)). This page says, for
each one, **what it does, its default value, and what raising or lowering
it does**. The file is read at launch: you edit, you relaunch.

Three rules for all of them:

- **Writing nothing keeps the default.** A line removed from the file goes
  back to its original value; to restore a setting, put it back at the
  value in the "default" column or delete the line.
- **A setting discourages or favors, it never forbids.** No number closes a
  branch or removes a track: the only bans are the bans themselves (`tb`,
  `ab`), which are not settings.
- **An absurd value is reset to its default, alone**, and reported on stderr
  at launch (a share outside 0–1, a negative half-life, a zero weight). The
  other lines stay as written.

The full commented template is the one forkstify writes on first run
(`src/config.rs`, `TEMPLATE`); on a machine installed before 2026-09-17,
copy it into the file by hand.

## The cooldowns (0012 §2)

What has just played steps back in the draw, then comes back with time.
Every cooldown has a **floor** (what is left of the weight on the same day)
and a **half-life** in days (the time to recover half of the rest).

| Setting | Default | What it does | Raise / lower |
|---|---|---|---|
| `track_cooldown_floor` | `0.1` | The share of its weight a track keeps on the day it played. | `1.0` = no cooldown at all, a track can come back the same evening. `0.0` = it does not come back that day (by the next one, it has already recovered a little). |
| `track_cooldown_half_life` | `7.0` | In days. At the default, a track played today is at 55 % of its weight a week later, 78 % at two, 94 % at a month. | Longer = tracks come back more slowly, the rotation widens. Shorter = you hear them again sooner. |
| `artist_cooldown_floor` | `0.3` | The share of their appeal an **artist** heard today keeps as a branch head. | `1.0` = the same artists can lead branch after branch. Lower = you cycle more between artists. |
| `artist_cooldown_half_life` | `4.0` | In days: the time for the artist to become a full branch head again. | Longer = an artist heard this week leads less; shorter = they come back quickly. |

The artist cooldown only touches the choice of **branch heads** and of the
`stay` branch; it does not remove that artist's tracks from the pool. The
track cooldown multiplies the track's weight inside its artist's pool.

## Taste (0018): `al` / `as`, `tl` / `ts`

Every artist carries a **weight** in `learned/`, starting at 1. "More often"
and "less often" multiply it; that weight then multiplies the weight of the
branches that start from that artist.

| Setting | Default | What it does | Raise / lower |
|---|---|---|---|
| `less_often` | `0.7` | What `as` / `ts` multiplies the artist's weight by. | Closer to 1 = a gentler gesture, it takes several to feel the difference. Lower = a single gesture clearly sets the artist aside. |
| `more_often` | `0` | What `al` / `tl` multiplies the weight by. `0` = the mirror of `less_often` (1 ÷ 0.7 ≈ 1.43), so that one `al` exactly undoes one `as`. | A value of its own (≥ 1) breaks the symmetry: `2.0` = "more often" weighs more than "less often". |
| `weight_floor` | `0.1` | The weight never goes below this value: an artist marked "less often" ten times stays possible. | Lower = you can almost erase an artist from the keyboard. Higher = the gesture tops out quickly. |
| `weight_ceiling` | `3.0` | The weight never goes above it: one key cannot run away with things. | Higher = a loved artist can dominate what gets proposed. |

## Familiarity (0001, 0014)

An artist's familiarity (0 to 1) comes from the imported library **or** from
plays inside forkstify, whichever is higher. That is what the comfort zone
reads: at the cocoon it attracts, wide open it repels.

| Setting | Default | What it does | Raise / lower |
|---|---|---|---|
| `plays_reference` | `5.0` | The number of (decayed) plays at which familiarity through listening reaches half. At 10 plays, 75 %; at 20, 94 %. | Lower = an artist becomes "familiar" quickly; higher = you have to have listened a lot. |
| `plays_half_life` | `182.5` | In days: the half-life of the play counters (six months, 0014). A play from a year ago counts for a quarter. | Shorter = familiarity follows what you are listening to now; longer = it has a memory. |

## The pool (0012 §1)

An artist's pool is the set of their drawable tracks, each with a weight;
the engine draws from it at weighted random. A top is worth 1: everything
else reads relative to it.

| Setting | Default | What it does | Raise / lower |
|---|---|---|---|
| `top_weight` | `1.0` | The weight of a top. The yardstick for the others; changing it alone amounts to changing all the others the other way. | Leave it at 1 unless there is a precise reason. |
| `liked_weight_cocoon` | `10.0` | The weight of a liked track (`tl`) at comfort 5. A liked track outranks the tops: tops are only the entry doors of a fresh fork (0018). | Higher = at the cocoon, you hear almost nothing but your liked tracks. Lower = the tops take their place back. |
| `liked_weight_open` | `2.0` | The same, at comfort 0; in between, the dial slides from one to the other. | Lower = wide open, a liked track counts for barely more than a top: you are after the unknown. |
| `door_weight` | `0.4` | The weight of a door (0011) that is not a top, when the branch does not go in its direction. | Higher = doors come out often, even with no direction. |
| `door_bonus` | `2.5` | What a door is multiplied by when the branch **does** go in its direction (one of the branch head's tags matches). | Higher = the door becomes the mandatory way into that direction. |
| `tail_weight` | `0.25` | The weight of **one** long-tail track (the rest of the discography), before the dial and familiarity cut it down. A tail has ten times more tracks than a card has tops: low per track, it weighs a lot in aggregate. | Higher = more deep cuts, even at middling comfort. Lower = the tail only shows up wide open, with familiar artists. |

The tail's share is `tail_weight × openness × familiarity`: nil at comfort 5
(nothing to tune here, that is the dial), and nil with an unknown artist,
who is led by their tops.

## Where the journey stands

The branch that **stays in the universe** aims at the middle of what has
been played. An evening drifts, though: starting at Can and ending at M83
by way of Paul McCartney, a plain average still sits back at the beginning,
and that branch kept proposing the artists of the first hour (Joel,
2026-09-23). So the middle follows the drift.

| Setting | Default | What it does | Raise / lower |
|---|---|---|---|
| `universe_half_life` | `5.0` | In **artists**: one played that far back counts half as much as the last one. At the default, roughly the last two branches say where the evening is. | Very large = the whole journey weighs the same, as it did before. Smaller = only the last few artists count, and the branch follows every turn. |

This changes **that branch alone**. The two others already looked at the
last branch only, never at the whole journey.

## The adventurous leap

Every round proposes a branch "through the graph" (the cards' links) and an
"adventurous" branch: an artist outside the graph, close in vector space.
Proximity is a cosine, from −1 to 1; in practice the useful neighbors sit
between 0.5 and 0.9. Every threshold has a value at the cocoon (comfort 5)
and one wide open (comfort 0); in between, the dial slides.

| Setting | Default | What it does | Raise / lower |
|---|---|---|---|
| `leap_floor_cocoon` | `0.80` | At comfort 5, below this proximity there is no leap outside the graph. | Higher = almost never an adventurous branch at the cocoon. Lower = it reaches further. |
| `leap_floor_open` | `0.60` | The same at comfort 0. It is also the floor below which `fw` goes looking for its "far away" ones. | Lower = wide open, you leap very far. |
| `leap_trust_cocoon` | `0.86` | At comfort 5, above this proximity a leap no longer needs a genre tag in common with the branch. | Lower = you trust the vectors alone sooner. |
| `leap_trust_open` | `0.70` | The same at comfort 0. | Likewise, wide open. |

Between floor and trust, a leap requires **a shared genre tag** (country and
decade do not count). A floor higher than the trust threshold makes no
sense: the trust threshold would never be used.

## What is not a setting, and why

- **Interface delays** (how long a toast lasts, the threshold for restarting
  a track, the seed offer): they decide nothing about what plays.
- **The shapes of the draw**: the number of candidates kept before the draw
  (`6` heads, `12` for `stay` and `fw`), the powers that deepen the
  weighting (`weight²` on the graph, `(proximity − 0.5)³` on the vectors),
  the fixed `4.0` weight of the `stay` branch, the `0.25`–`2` range of what
  comfort does to a familiarity. Those are mechanics, not numbers; opening
  one up is one more line in `[tuning]` and one more line here.
