# Research: compass pulse arrow

**Recommendation:** Move the always-on in-world arrow to a small edge-anchored chevron on the player cell's border (not centered in the next corridor cell), ~28px, static (no continuous pulse), hidden when the active target is already inside the 3-cell fog radius. Derive direction from the BFS next step, not from sign(dx/dy).

## Current state (measured in `index.html`)

- Arrow is centered in the target-adjacent cell (lines 1384-1397): dark disc r=24 (Ø48px in a 56px cell, `CS=56`), 46px emoji, fully opaque, drawn every frame even though named "pulse" — it does not pulse.
- It covers exactly the cell the player must move into next, competing with traps, aim wedges (lines 1360-1381), and the aim `blink` — occlusion, not just screen contrast. `directionToTarget()` (lines 443-459) uses raw position sign, so in a maze it can point into a wall.

## Design constraints from primary sources

- **Contrast floor:** meaningful icons/graphics need ≥3:1 against adjacent colors; text needs ≥4.5:1 (3:1 large) — a translucent/greyed-out arrow still needs a halo that clears 3:1, or it fails readability [W3C 1.4.11](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html) [W3C 1.4.3](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html).
- **Don't encode essential info by color alone:** direction already uses shape (chevron/arrow glyph), which is the correct fallback [W3C 1.4.1](https://www.w3.org/TR/WCAG22/#use-of-color) [Apple HIG: Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility.md).
- **Motion is purposeful or it is a cost:** Apple HIG says add motion purposefully, keep feedback brief/precise, make motion optional and never the only channel [Apple HIG: Motion](https://developer.apple.com/design/human-interface-guidelines/motion.md); Material: "use motion to make a UI expressive and easy to use" [Material M3](https://m3.material.io/styles/motion/overview).
- **Respect reduced motion:** `matchMedia('(prefers-reduced-motion: reduce)')` gates scaling/panning animation for vestibular safety [MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion); canvas-side, the equivalent is skipping opacity/slide pulses [MDN rAF](https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame).
- **Flash budget:** nothing flashes >3 times per second above thresholds — a 0.5s alternate blink is far under this, but keep any on-change pulse to a single fade, not a loop [W3C 2.3.1](https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html) [W3C 2.3.3](https://www.w3.org/TR/WCAG22/#animation-from-interactions).
- **Translucency is cheap, structure is what matters:** `ctx.globalAlpha` (0-1, default 1) applies to shapes/images pre-drawn [MDN](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/globalAlpha) — cheap to fade, but alpha alone doesn't fix that the arrow sits in the next gameplay cell; only relocation does.

## Options: intrusiveness vs readability

| Option | Intrusiveness | Readability | Verdict |
|---|---|---|---|
| Status quo (Ø48, opaque, in next cell) | Highest: hides next-cell danger/walls | Best at a glance in fog | Reject — worst occlusion/eye-draw |
| Smaller + translucent, same spot | Lower | Same cell still hidden; risks <3:1 if faded too far | Half-fix only |
| Continuous pulsing | High: perpetual motion in peripheral vision, Apple warns it distracts | High salience, over-signals | Reject as continuous; allow ≤1 fade per target change |
| Edge-anchored chevron on player border | Low: no in-world cell covered | High: fixed glance point, fog-proof | **Adopt** — best trade |
| Distance-fading (fade as target nears) | Low | Inverted: guide fades exactly when fog judgment is hardest | Adopt in reverse: hide when target within `FOG_RADIUS` |
| Remove entirely (BFS hint on keypress) | Zero | Loss of continuous orientation in fog; exploration suffers | Add as opt-in helper, not replacement |

## Implementation notes (canvas, no deps)

- Anchor: `C + CS/2 + (dx|dy)*(CS/2-4)` — inside player cell edge, z-order below aim wedges/vignette.
- Size: 24-28px glyph, Ø~32px halo; verify ≥3:1 (gold `#ffd166` on dark halo/scene passes; test against lake/swamp underlay, not just fog).
- Alpha: 0.85 flat; on target switch only, 0→0.85 fade over ~800ms (single, not looped); static when `prefers-reduced-motion`.
- Hide when `dist(player, activeTarget) <= CFG.FOG_RADIUS` (BFS dist already in `computeVisibility`), keep 🏆 when on-target.
- Direction: reuse existing `bfs`/`prev` next-step, not sign differences, to avoid wall-pointing.
- Check: ensure the chevron never overlaps the shooter/boss aim arc; toggle with existing help/pause state.
