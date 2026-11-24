# Story, spec, storyboard and review

Read this before any video over about 15 s that tells something: a promo, a launch, an explainer or a recap. Skip it for captions on footage, a logo sting or a one-line title, where the request is the story. If the person gave a script or a storyboard, follow theirs.

## The one thing

- **What it is**, in one line. "Sekarang bisa pesan kue lewat WhatsApp" for a launch. "Menu baru: Es Kopi Pandan" for a promo.
- **What the viewer remembers** afterwards, in the viewer's words, not as a slogan.
- **One viewer doing one real task**, from the need to the result, with one real thing travelling through it: an order, a dish, a photo, a price.
- Keep facts (what it does, prices, hours, real labels) apart from creative choices (angle, tone, look). Every claim on screen traces to the brief or the person's own words. What the brief doesn't say stays unsaid. If a screen or photo is missing, show a labelled placeholder ("Foto: tampak depan toko") and say so in your reply.

## The hook

The first two seconds are the densest of the film, and frame 0 is the thumbnail. Open on the result already there (the steaming bowl, the finished cake), a real moment that creates the need, or a question with the answer's input inside it. Never open on black, a logo alone, or typing into a box with nothing else on screen.

## Beats

| Beat | 10–15 s Reel | ~35 s launch | Job |
|---|---|---|---|
| Hook | 0–1.5 s | ~8% | The result or the need, already on screen |
| Tension | (skip) | ~12% | The old way failing: queue, wait, sold out |
| Reveal | 1.5–4 s | ~8% | The product or offer named as a title |
| Proof | 4–9 s | ~40% | The real thing in use, as 3–6 verbs: pilih, pesan, ambil, nikmati |
| Payoff | (merged with proof) | ~20% | The biggest visual moment, once, 60–80% in |
| Claim | (merged with end card) | ~8% | The one thing in one line |
| End card | last 2–3 s | ~8% | The mark carried in, the name and the action land and hold 2–3 s |

- Visit several "worlds" (counter, kitchen, table, the phone) instead of one dark canvas with small things floating in it. Enter each world through an object already on screen.
- The brand lands once, with weight, usually on the end card. It is never docked small in a corner the whole time.
- Show, don't list: no bullet slides and no three cards in a row.

## The spec

Before writing any scene, write the film as a brief to yourself, with a number wherever you would have written an adjective. Keep it in your plan, or as `SPEC.md` beside `index.html`.

```text
GOAL:    what the viewer feels or does at the end, one sentence
OUTPUT:  W x H, fps
TIME:    seconds x fps = frames; BPM and frames per beat when there is music
LOOK:    ground, ink, one accent and where it may appear (from frame.md);
         display face + weight + size in px, body face the same; the medium (photographs, paper, product UI)
MOTION:  eases by role, springs as a damping ratio, durations and staggers in frames
SHOTS:   one line per shot: frame range, the one hero move, what the viewer reads (with its window), the sound
BANNED:  the defaults below, plus anything this film must not become
```

| The word | Becomes |
|---|---|
| premium | a material with numbers: the cake on a seamless warm-grey sweep with a soft contact shadow, 2% grain |
| modern | one camera move per shot on one ease, display type at 9% of the frame height with -2% tracking |
| clean | a grid and a ration: 96 px margins, the accent on one word per shot |
| dynamic | cuts on the beat, a new layout every bar, one hero move per shot |
| playful | springs at damping 0.45–0.6 on position and scale, never on opacity |

**Composition numbers:** the hero of a shot fills at least 40% of the frame's height or width. Secondary moves travel 50–70% as far, start 100–200 ms after the hero move, and everything else holds still. Text meant to be read is at least 28 px at 1080 (2.6% of the short side).

**Banned by default:** a dark or navy void with small floating elements; glow, bloom, neon, lens flares and particles; purple or blue gradients and glass cards; the cream-paper serif look with "01 / 02" labels and mono caps; everything centred; everything fading in at once; text that moves while it is read; corner brackets and HUD readouts as decoration. Use one of these only when the brand's own material does.

## The storyboard

For a film the person should approve first, show the shots before animating. Write `STORYBOARD.md` beside `index.html`. The frontmatter holds `format`, `duration`, `message` and `arc`. Each frame gets an H2 heading with its time and these bullets:

```markdown
## Frame 1: The hook (0-1.5s)
- scene: Es Kopi Pandan poured over ice, top-down, the green swirl fills the glass
- reads: "Es Kopi Pandan" lands (0.4-1.5s)
- sound: bed enters on its strongest bar
- status: outline
```

`status` moves from `outline` to `built` (the layout is posed) to `animated`. Pose each shot at its key moment with real words, fonts and colours, so the Studio can show it as a still. When the person comments on a frame, change only that frame. Once they say go, animate from the approved stills.

## Sound

- A film over about 15 s that tells something gets a music bed and sounds on visible actions, unless the person wants silence or it will play muted. A cafe TV menu plays muted, so the pictures and type carry it alone.
- Use the person's own track first. Don't generate music in code. If there is no music source, deliver silent and say so in one line.
- Use sound effects sparingly, each with a cause on screen: a pop when a price lands, a whoosh on a cut, a riser once into the payoff. Rotate two or three takes; the same whoosh on every cut is a tell.
- Music under a voice sits around volume 0.14. The mix for social is around −16 to −14 LUFS, with peaks at −1 dBTP or lower. Say that sound was measured, not heard.
- Many people watch Reels muted, so speech is captioned and the picture carries the hook without sound.

## Review before you hand off

Read the frames as a stranger seeing them once, at full speed, on a phone.

1. Describe each beat frame in one sentence, naming every object. An object you would have to explain is a problem.
2. Do the frames tell the story in order, each one causing the next? With the words covered, do the pictures still tell it?
3. Can every line be read at phone width (360 px)? Is each line on screen long enough for its words (0.5 s + 0.3 s a word)? Is there a stretch of more than 3 s with nothing changing?
4. Is frame 0 empty? It is the thumbnail on social players.
5. Which banned defaults or generated-looking moves (`motion.md`) are on screen?

Write each problem as one line: `t=4.2s · where · problem → exact change`. Rank them P0 (breaks the read or the contract), P1 (weakens the film) and P2 (polish). Fix every P0 and the P1s worth their time: text first, then timing, then detail. Never score the film out of 10. In your reply, name the creative calls you made, what you checked, the weakest shot, and any warning you kept on purpose.

Adapted from hyperframes (Apache-2.0, HeyGen). Changed: rewritten for Nomi's video compositions.
