//! The fonts text is drawn in: Inter for Latin letters and digits, MiSans VF for Chinese
//! (ADR 0003). The MiSans subset holds only the characters the game shows.

/// MiSans VF cut down to the characters the game shows.
pub const MISANS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/MiSansVF-subset.ttf"));

/// The fonts the app registers. On native, registering any font replaces Rune's built-in
/// Inter, so Inter is registered here too; the web keeps Rune's bundled Inter beside these.
#[cfg(not(target_arch = "wasm32"))]
pub const LIST: &[&[u8]] = &[
    include_bytes!("../assets/fonts/Inter/Inter-Regular.ttf"),
    include_bytes!("../assets/fonts/Inter/Inter-Medium.ttf"),
    include_bytes!("../assets/fonts/Inter/Inter-SemiBold.ttf"),
    MISANS,
];
#[cfg(target_arch = "wasm32")]
pub const LIST: &[&[u8]] = &[MISANS];

/// The sans and mono slots: Inter, then MiSans for what Inter lacks. MiSans comes before
/// any system CJK font, so Chinese looks the same on every machine.
pub const FAMILIES: (&str, &str) = ("Inter, MiSans VF", "Inter, MiSans VF");

#[cfg(test)]
mod tests {
    use skrifa::{FontRef, MetadataProvider, string::StringId};

    use super::*;
    use crate::strings::{Lang, S};

    fn family(font: &FontRef) -> String {
        font.localized_strings(StringId::FAMILY_NAME).english_or_first().map(|s| s.to_string()).unwrap_or_default()
    }

    /// The family of the first registered font that draws `c`.
    fn drawn_by(fonts: &[&[u8]], c: char) -> Option<String> {
        fonts.iter().map(|f| FontRef::new(f).unwrap()).find(|f| f.charmap().map(c).is_some()).map(|f| family(&f))
    }

    #[test]
    fn latin_and_digits_are_inter_and_chinese_is_misans() {
        let spec = crate::spec();
        let sans: Vec<&str> = spec.families.0.split(',').map(str::trim).collect();
        assert_eq!(sans, ["Inter", "MiSans VF"], "Inter first, so only what it lacks reaches MiSans");
        for f in spec.fonts {
            FontRef::new(f).expect("every registered font parses");
        }
        // Registering any font on native replaces Rune's built-in Inter, so it is registered too.
        #[cfg(not(target_arch = "wasm32"))]
        for c in "Mines left: 0123456789".chars().filter(|c| !c.is_whitespace()) {
            assert_eq!(drawn_by(spec.fonts, c).as_deref(), Some("Inter"), "{c:?}");
        }
        for c in "剩余雷数：简体中文".chars() {
            assert_eq!(drawn_by(spec.fonts, c).as_deref(), Some("MiSans VF"), "{c:?}");
        }
    }

    #[test]
    fn misans_draws_every_character_the_zh_cn_table_shows() {
        let font = FontRef::new(MISANS).expect("MiSans parses");
        assert_eq!(family(&font), "MiSans VF");
        let charmap = font.charmap();
        let shown = S::ALL.iter().map(|k| k.get(Lang::ZhCn)).chain([Lang::ZhCn.endonym()]);
        for s in shown {
            for c in s.chars().filter(|c| !c.is_ascii()) {
                assert!(charmap.map(c).is_some(), "MiSans has no {c:?} (in {s:?})");
            }
        }
    }
}
