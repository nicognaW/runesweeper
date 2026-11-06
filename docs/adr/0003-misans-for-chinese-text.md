# Chinese text uses MiSans, committed under its own licence

zh-CN text is drawn with Xiaomi's MiSans VF; Latin letters and digits stay in Inter. MiSans is not open source: its agreement forbids redistributing the font on its own or changing its glyphs, and requires the software to say it uses MiSans. We commit the official file with the agreement beside it in `assets/fonts/MiSans/`, cut a subset of only the characters the game uses at build time, and compile that subset into the native and wasm binaries. Settings carries the one credit line the licence requires. The font is excluded from the project's MIT licence.

## Consequences

- Distributors with free-software rules (Debian, Flathub) will treat the font as non-free and may need to swap it.
- Subsetting is our reading of a licence that is silent on it (it changes no glyph shapes); Xiaomi could disagree.
