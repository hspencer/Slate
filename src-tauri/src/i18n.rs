//! Textes natifs (menus, filtres de dialogue) : mêmes catalogues JSON que le
//! frontend (`frontend-dist/locales`), une seule source de vérité.
use std::collections::HashMap;
use std::sync::OnceLock;

const CATALOGS: [(&str, &str); 3] = [
    ("fr", include_str!("../frontend-dist/locales/fr.json")),
    ("en", include_str!("../frontend-dist/locales/en.json")),
    ("es", include_str!("../frontend-dist/locales/es.json")),
];

/// Repli par complétude des catalogues : es → en → fr (voir specs/i18n.allium).
fn chain(locale: &str) -> &'static [&'static str] {
    match locale {
        "fr" => &["fr"],
        "en" => &["en", "fr"],
        _ => &["es", "en", "fr"],
    }
}

/// Variante régionale (es-CL, fr_CA…) → langue de base ; non gérée → es (défaut).
fn base(tag: &str) -> &'static str {
    match tag.to_lowercase().split(['-', '_']).next() {
        Some("fr") => "fr",
        Some("en") => "en",
        _ => "es",
    }
}

/// Langue des textes natifs, lue au lancement (réglage persisté par Slate).
pub fn locale() -> &'static str {
    #[cfg(target_os = "macos")]
    let tag = crate::mac_print::effective_ui_language();
    #[cfg(not(target_os = "macos"))]
    let tag = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_default();
    base(&tag)
}

fn catalogs() -> &'static HashMap<&'static str, HashMap<String, String>> {
    static CELL: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
    CELL.get_or_init(|| {
        CATALOGS
            .iter()
            .map(|(l, json)| {
                let map: HashMap<String, serde_json::Value> =
                    serde_json::from_str(json).unwrap_or_default();
                let strings = map
                    .into_iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                    .collect();
                (*l, strings)
            })
            .collect()
    })
}

/// Texte natif pour `key` ; clé brute si absente de toute la chaîne de repli.
pub fn tr(key: &str) -> String {
    let all = catalogs();
    chain(locale())
        .iter()
        .find_map(|l| all.get(l).and_then(|c| c.get(key)))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_and_dialog_keys_exist_in_every_locale() {
        let all = catalogs();
        let fr = &all["fr"];
        for locale in ["fr", "en", "es"] {
            let cat = &all[locale];
            for key in fr.keys().filter(|k| k.starts_with("menu.") || k.starts_with("dialog.")) {
                assert!(cat.contains_key(key), "{locale} sans {key}");
            }
        }
        assert!(fr.keys().any(|k| k.starts_with("menu.")));
    }

    #[test]
    fn base_resolves_regional_variants_and_defaults_to_es() {
        assert_eq!(base("fr_CA"), "fr");
        assert_eq!(base("en-GB"), "en");
        assert_eq!(base("es-CL"), "es");
        assert_eq!(base("de-DE"), "es");
    }
}
