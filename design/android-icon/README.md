# Android voice cursor icon

Generated with the built-in image generation tool on 2026-09-09.

- `voice-cursor-master.png`: unmodified generated artwork, 1254 × 1254.
- `voice-cursor-play-512.png`: mechanically resized and encoded as 512 × 512, sRGB, 8-bit RGBA PNG. Full square, no outer corner mask or drop shadow.
- Concept: mint voice waveform joining a text insertion cursor, matching the Android app's charcoal and mint palette. Generated colors may vary slightly from requested tokens.

Applied across Android and desktop on 2026-09-09:

- Android: separate transparent foreground and charcoal background, adaptive launcher resource, Android 13 themed monochrome alpha, legacy PNG fallback.
- macOS: rounded tile ICNS with transparent outer margins; menu bar uses a transparent 40 px template image whose alpha macOS tints for light/dark appearance.
- Windows: multi-resolution ICO; Linux: PNG sizes provided through existing Tauri bundle configuration. Their native runtime verification remains pending.
- `voice-cursor-foreground.png`: built-in image generation extraction of the approved artwork, with transparency. Extraction prompt: preserve the three shapes, spacing and proportions; remove only the charcoal background, retain mint, no redesign or shadow.
- `src-tauri/icons/app-icon.svg` is unused upstream source artwork, retained for upstream history; it is not the source for these exports.

## Rebuilding exports

From the repository root on macOS, run `swift scripts/h-export-icons.swift` to export the Android foreground, menu bar RGBA/PNG, and macOS iconset. Then `iconutil -c icns design/android-icon/macos.iconset -o src-tauri/icons/icon.icns`.

For Windows/Linux, run the installed Tauri CLI `tauri icon design/android-icon/voice-cursor-master.png -o <temporary-directory>` and copy only `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`, and `icon.ico` into `src-tauri/icons`. Do not overwrite the macOS ICNS with the square variant. No AI calls are needed to reproduce these exports.

Official references:
- https://developer.android.com/distribute/google-play/resources/icon-design-specifications
- https://developer.android.com/develop/ui/compose/system/icon_design_adaptive

## Generation prompt

Use case: logo-brand. Generate one finished Android app icon artwork for H-OpenTypeless, a refined native voice-to-text keyboard. Square 512 x 512 pixel PNG requested. Full bleed perfectly uniform charcoal #1A1A1A background to all four square corners. In the center, a single beautifully balanced, bold mint #2ABBA7 symbol: three softly rounded vertical audio waveform bars of varying height, with the rightmost taller stroke subtly shaped like a text insertion cursor, forming one cohesive voice-to-typing identity. Extremely clean flat vector-like geometry, precise optical spacing, confident thick strokes, elegant restrained contemporary Android product design, instantly readable at small launcher sizes. Keep the entire mint mark inside the central circle whose diameter is 60 percent of the canvas; generous intentional negative space. No written words, letters, labels, tiny keyboard keys, sparkles, badges, borders, gradients, textures, bevels, glow, drop shadows, mockup, device frame, rounded outer tile or presentation sheet. A single production icon, straight-on, no surrounding canvas. All background is opaque. Output only the icon.
