# Interstellar Forager: The Enchanted Forest of Naibihpma (15 FPS VGA)

A 15 FPS, 64-color VGA pixel-pure (`400x300` logical canvas presented at exact `Nx` integer multiples in borderless fullscreen, with `[F2]` toggleable CRT scanlines) **interstellar foraging & tavern cooking roguelike** written in pure Rust (`winapi`), combining:
- **Phase 1 — Tactical Stealth-Forager & Star-Pond Diver:** Deliberate stealth-stalking, automatic obstacle crawling & gap-leaping, overhead harpoon grapple-winching (`[UP]+[X]`), bioluminescent lure-trapping (`[C]`), soporific tranquilizer darts (`[Z]/[SPACE]`), 8-way buoyant swimming, *Dave the Diver*-style $\text{O}_2$-is-Health & Timer economy, and interlocking world-state sluice/counterweight puzzles.
- **Phase 2A — Pre-Service Menu Combiner:** Pair harvested `1*–3*` specimens into curated menu dishes before opening the tavern doors.
- **Phase 2B — Live Side-View Tavern Service:** Stoke the hearth, pour Spore-Tea (`[C]`), and serve impatient amphibian aristocrats at the bar.
- **Phase 3 — Tavern & Foraging Rig Upgrade Shop:** Invest nightly earnings into kitchen and diving rig upgrades.

See:
- [docs/18_level_progression.md](docs/18_level_progression.md) for the full 18-level, 3-act campaign blueprint.
- [../visual_and_rendering_principles.md](../visual_and_rendering_principles.md) for the `400x300` 64-color VGA rendering, exact integer scaling, CRT scanline (`[F2]`), and 15 FPS simulation specifications.

## Building & Running

```powershell
cargo run --release
```

*(Optional)* Place `.wav` tracks in `assets/music/` for the in-game soundtrack; the game runs cleanly without them if `assets/music/` is absent.
