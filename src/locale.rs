//! The system's language, for the game's language until the player picks one: the user's
//! preferred UI language on macOS and Windows, the locale variables elsewhere, the browser's
//! language on the web.

use crate::strings::Lang;

/// The game's language on a first launch, before one is picked in Settings.
pub fn system() -> Lang {
    system_tag().map_or(Lang::En, |t| Lang::from_locale(&t))
}

#[cfg(target_arch = "wasm32")]
fn system_tag() -> Option<String> {
    web_sys::window()?.navigator().language()
}

/// `LANGUAGE`'s first entry, then `LC_ALL`, `LC_MESSAGES` and `LANG`, as gettext reads them.
#[cfg(not(target_arch = "wasm32"))]
fn env_tag() -> Option<String> {
    locale_vars(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
}

/// The locale tag the variables `var` reads name, if any.
#[cfg(not(target_arch = "wasm32"))]
fn locale_vars(var: impl Fn(&str) -> Option<String>) -> Option<String> {
    let language = var("LANGUAGE").and_then(|v| v.split(':').next().map(str::to_owned)).filter(|v| !v.is_empty());
    language.or_else(|| var("LC_ALL")).or_else(|| var("LC_MESSAGES")).or_else(|| var("LANG"))
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "macos"), not(windows)))]
fn system_tag() -> Option<String> {
    env_tag()
}

/// The first of the user's preferred languages (System Settings → Language & Region); apps
/// opened from Finder get no `LANG`.
#[cfg(target_os = "macos")]
fn system_tag() -> Option<String> {
    use std::ffi::{CStr, c_char, c_void};
    type CfRef = *const c_void;
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFLocaleCopyPreferredLanguages() -> CfRef;
        fn CFArrayGetCount(array: CfRef) -> isize;
        fn CFArrayGetValueAtIndex(array: CfRef, i: isize) -> CfRef;
        fn CFStringGetCString(s: CfRef, buf: *mut c_char, size: isize, encoding: u32) -> u8;
        fn CFRelease(r: CfRef);
    }
    const UTF8: u32 = 0x0800_0100;
    // SAFETY: the array is owned here (a Copy function) and released once; the string is
    // borrowed from it and copied out before then.
    let preferred = unsafe {
        let langs = CFLocaleCopyPreferredLanguages();
        if langs.is_null() {
            None
        } else {
            let mut out = None;
            if CFArrayGetCount(langs) > 0 {
                let mut buf = [0 as c_char; 64];
                if CFStringGetCString(CFArrayGetValueAtIndex(langs, 0), buf.as_mut_ptr(), buf.len() as isize, UTF8) != 0 {
                    out = CStr::from_ptr(buf.as_ptr()).to_str().ok().map(str::to_owned);
                }
            }
            CFRelease(langs);
            out
        }
    };
    preferred.or_else(env_tag)
}

/// The first of the user's preferred UI languages (Settings → Time & language).
#[cfg(windows)]
fn system_tag() -> Option<String> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserPreferredUILanguages(flags: u32, count: *mut u32, buf: *mut u16, len: *mut u32) -> i32;
    }
    const MUI_LANGUAGE_NAME: u32 = 0x8;
    let (mut count, mut buf, mut len) = (0u32, [0u16; 256], 256u32);
    // SAFETY: `buf` holds `len` UTF-16 units, which the call fills with NUL-separated names.
    let ok = unsafe { GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, buf.as_mut_ptr(), &mut len) } != 0;
    let first = buf.iter().position(|&u| u == 0).unwrap_or(0);
    let tag = (ok && count > 0 && first > 0).then(|| String::from_utf16_lossy(&buf[..first]));
    tag.or_else(env_tag)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn first_launch(vars: &[(&str, &str)]) -> Lang {
        locale_vars(|k| vars.iter().find(|(n, _)| *n == k).map(|(_, v)| v.to_string()).filter(|v| !v.is_empty())).map_or(Lang::En, |t| Lang::from_locale(&t))
    }

    #[test]
    fn a_chinese_locale_picks_zh_cn_on_first_launch() {
        assert_eq!(first_launch(&[("LANG", "zh_CN.UTF-8")]), Lang::ZhCn);
        assert_eq!(first_launch(&[("LANG", "en_US.UTF-8"), ("LANGUAGE", "zh_TW:en")]), Lang::ZhCn, "LANGUAGE's first entry comes first");
        assert_eq!(first_launch(&[("LANG", "zh_CN.UTF-8"), ("LC_ALL", "en_GB.UTF-8")]), Lang::En, "LC_ALL over LANG");
        assert_eq!(first_launch(&[("LANG", "en_US.UTF-8"), ("LC_MESSAGES", "zh_CN.UTF-8")]), Lang::ZhCn, "LC_MESSAGES over LANG");
        assert_eq!(first_launch(&[("LANG", "C")]), Lang::En);
        assert_eq!(first_launch(&[("LANGUAGE", ""), ("LANG", "fr_FR.UTF-8")]), Lang::En);
        assert_eq!(first_launch(&[]), Lang::En, "no locale at all");
    }
}
