//! Embeds the Windows icon, and cuts MiSans VF down to the characters the game shows
//! (ADR 0003): every non-ASCII character in the string table's literals, so a new zh-CN
//! string needs no font step. Inter draws ASCII.

use std::collections::BTreeSet;
use std::path::Path;

const MISANS: &str = "assets/fonts/MiSans/MiSansVF.ttf";
const STRINGS: &str = "src/strings.rs";

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed={MISANS}");
    println!("cargo:rerun-if-changed={STRINGS}");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico").set("ProductName", "Runesweeper").set("FileDescription", "Runesweeper");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded: {e}");
        }
    }
    let source = std::fs::read_to_string(STRINGS).expect("read the string table");
    let chars = shown_chars(&source);
    let font = std::fs::read(MISANS).expect("read MiSans VF");
    let subset = subset(&font, &chars);
    let out = Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("MiSansVF-subset.ttf");
    std::fs::write(out, subset).expect("write the MiSans subset");
}

/// The non-ASCII characters inside the string literals of `source`, skipping comments.
fn shown_chars(source: &str) -> BTreeSet<char> {
    let mut out = BTreeSet::new();
    let mut it = source.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '/' if it.peek() == Some(&'/') => {
                for c in it.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            // A char literal such as '"' would open a string; none of them is a quote.
            '\'' => {
                if let Some('\\') = it.next() {
                    it.next();
                }
            }
            '"' => {
                while let Some(c) = it.next() {
                    match c {
                        '\\' => {
                            it.next();
                        }
                        '"' => break,
                        c if !c.is_ascii() => {
                            out.insert(c);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn subset(font: &[u8], chars: &BTreeSet<char>) -> Vec<u8> {
    use fontcull_klippa::{Plan, SubsetFlags, subset_font};
    use fontcull_skrifa::{FontRef, GlyphId, Tag};
    use fontcull_write_fonts::read::collections::IntSet;
    use fontcull_write_fonts::types::NameId;

    let font = FontRef::new(font).expect("parse MiSans VF");
    let unicodes: IntSet<u32> = chars.iter().map(|&c| c as u32).collect();
    let features: IntSet<Tag> = fontcull_klippa::DEFAULT_LAYOUT_FEATURES.iter().copied().collect();
    // The game sets no vertical text.
    let drop: IntSet<Tag> = [Tag::new(b"vhea"), Tag::new(b"vmtx"), Tag::new(b"DSIG")].into_iter().collect();
    // Every name record: the copyright notice stays with the font, and the axis and
    // feature names fvar and STAT point at.
    let plan = Plan::new(
        &IntSet::<GlyphId>::empty(),
        &unicodes,
        &font,
        SubsetFlags::default(),
        &drop,
        &IntSet::all(),
        &features,
        &IntSet::<NameId>::all(),
        &IntSet::<u16>::all(),
    );
    subset_font(&font, &plan).expect("subset MiSans VF")
}
