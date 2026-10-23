use std::collections::{BTreeMap, BTreeSet};

use fluent_syntax::ast;
use intl_pluralrules::{PluralCategory, PluralRuleType, PluralRules};

use super::*;

/// Message ids with the variables each one uses
fn messages(locale: Locale) -> BTreeMap<String, BTreeSet<String>> {
    let resource = fluent_syntax::parser::parse(locale.source())
        .unwrap_or_else(|(_, e)| panic!("{}: {e:?}", locale.tag()));
    let mut out = BTreeMap::new();
    for entry in resource.body {
        if let ast::Entry::Message(m) = entry {
            let mut vars = BTreeSet::new();
            if let Some(p) = &m.value {
                pattern_vars(p, &mut vars);
            }
            out.insert(m.id.name.to_string(), vars);
        }
    }
    out
}

fn pattern_vars(p: &ast::Pattern<&str>, out: &mut BTreeSet<String>) {
    for e in &p.elements {
        if let ast::PatternElement::Placeable { expression } = e {
            expression_vars(expression, out);
        }
    }
}

fn expression_vars(e: &ast::Expression<&str>, out: &mut BTreeSet<String>) {
    match e {
        ast::Expression::Select { selector, variants } => {
            inline_vars(selector, out);
            for v in variants {
                pattern_vars(&v.value, out);
            }
        }
        ast::Expression::Inline(i) => inline_vars(i, out),
    }
}

fn inline_vars(e: &ast::InlineExpression<&str>, out: &mut BTreeSet<String>) {
    match e {
        ast::InlineExpression::VariableReference { id } => {
            out.insert(id.name.to_string());
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline_vars(a, out);
            }
        }
        ast::InlineExpression::Placeable { expression } => expression_vars(expression, out),
        _ => {}
    }
}

/// Select expressions on a variable, with their variant keys and default
fn selects(locale: Locale) -> Vec<(String, BTreeSet<String>, String)> {
    let resource = fluent_syntax::parser::parse(locale.source()).unwrap();
    let mut out = Vec::new();
    for entry in resource.body {
        let ast::Entry::Message(m) = entry else {
            continue;
        };
        let Some(p) = &m.value else { continue };
        for e in &p.elements {
            if let ast::PatternElement::Placeable {
                expression: ast::Expression::Select { variants, .. },
            } = e
            {
                let mut keys = BTreeSet::new();
                let mut default = String::new();
                for v in variants {
                    let key = match &v.key {
                        ast::VariantKey::Identifier { name } => name.to_string(),
                        ast::VariantKey::NumberLiteral { value } => value.to_string(),
                    };
                    if v.default {
                        default = key.clone();
                    }
                    keys.insert(key);
                }
                out.push((m.id.name.to_string(), keys, default));
            }
        }
    }
    out
}

/// The rules Fluent itself picks, narrowing a tag such as `es-419` to the
/// nearest language the plural data has
fn plural_rules(l: Locale) -> PluralRules {
    let id: LanguageIdentifier = l.tag().parse().unwrap();
    let english: LanguageIdentifier = "en".parse().unwrap();
    let available = PluralRules::get_locales(PluralRuleType::CARDINAL);
    let chosen = fluent_langneg::negotiate_languages(
        &[id],
        &available,
        Some(&english),
        fluent_langneg::NegotiationStrategy::Lookup,
    )[0]
    .clone();
    assert!(
        l == Locale::En || chosen != english,
        "{} has no plural rules",
        l.tag()
    );
    PluralRules::create(chosen, PluralRuleType::CARDINAL).unwrap()
}

fn category_name(c: PluralCategory) -> &'static str {
    match c {
        PluralCategory::ZERO => "zero",
        PluralCategory::ONE => "one",
        PluralCategory::TWO => "two",
        PluralCategory::FEW => "few",
        PluralCategory::MANY => "many",
        PluralCategory::OTHER => "other",
    }
}

/// Fluent wraps arguments in bidi isolates, which tests compare without
fn plain(s: String) -> String {
    s.chars()
        .filter(|c| !matches!(c, '\u{2068}' | '\u{2069}'))
        .collect()
}

#[test]
fn every_catalog_loads() {
    for l in Locale::ALL {
        Catalog::new(l);
    }
}

#[test]
fn every_catalog_has_the_english_messages_and_arguments() {
    let english = messages(Locale::En);
    for l in Locale::ALL {
        let other = messages(l);
        assert_eq!(
            other.keys().collect::<Vec<_>>(),
            english.keys().collect::<Vec<_>>(),
            "{} messages",
            l.tag()
        );
        for (id, vars) in &english {
            assert_eq!(&other[id], vars, "{} {id} arguments", l.tag());
        }
    }
}

#[test]
fn plural_selects_cover_every_category_whole_numbers_reach() {
    let samples: Vec<u64> = (0..=200).chain([1000, 1_000_000]).collect();
    for l in Locale::ALL {
        let rules = plural_rules(l);
        for (message, keys, default) in selects(l) {
            for n in &samples {
                let category = category_name(rules.select(*n).unwrap());
                assert!(
                    keys.contains(category) || default == category,
                    "{} {message} has no [{category}] form for {n}",
                    l.tag()
                );
            }
        }
    }
}

#[test]
fn plural_forms_follow_each_language() {
    let say = |l: Locale, n: i64| {
        plain(Catalog::new(l).text_with("details-kills", &[("count", n.into())]))
    };
    assert_eq!(say(Locale::En, 1), "1 kill");
    assert_eq!(say(Locale::En, 2), "2 kills");
    for (n, word) in [
        (1, "убийство"),
        (2, "убийства"),
        (5, "убийств"),
        (11, "убийств"),
        (21, "убийство"),
        (22, "убийства"),
        (25, "убийств"),
    ] {
        assert_eq!(say(Locale::Ru, n), format!("{n} {word}"));
    }
    assert_eq!(say(Locale::Pl, 22), "22 zabójstwa");
    assert_eq!(say(Locale::Pl, 25), "25 zabójstw");
    assert_eq!(say(Locale::Lt, 21), "21 nužudymas");
    assert_eq!(say(Locale::Lt, 10), "10 nužudymų");
    assert_eq!(say(Locale::Ar, 2), "عمليتا قتل");
    assert_eq!(say(Locale::Ar, 3), "3 عمليات قتل");
    assert_eq!(say(Locale::Ar, 11), "11 عملية قتل");
    assert_eq!(
        say(Locale::Fr, 1_000_000),
        "1\u{202f}000\u{202f}000 éliminations"
    );
    assert_eq!(say(Locale::Es419, 1), "1 baja");
    assert_eq!(say(Locale::PtBr, 0), "0 abate");
}

/// Counts that reach every plural form of the languages with several
#[test]
fn plurals_are_right_for_counts_of_every_form() {
    let say = |l: Locale, n: i64| {
        plain(Catalog::new(l).text_with("details-kills", &[("count", n.into())]))
    };
    let counts = [0, 1, 2, 5, 11, 21, 22, 25];
    let expected: [(Locale, [&str; 8]); 5] = [
        (
            Locale::Ru,
            [
                "0 убийств",
                "1 убийство",
                "2 убийства",
                "5 убийств",
                "11 убийств",
                "21 убийство",
                "22 убийства",
                "25 убийств",
            ],
        ),
        (
            Locale::Uk,
            [
                "0 вбивств",
                "1 вбивство",
                "2 вбивства",
                "5 вбивств",
                "11 вбивств",
                "21 вбивство",
                "22 вбивства",
                "25 вбивств",
            ],
        ),
        (
            Locale::Pl,
            [
                "0 zabójstw",
                "1 zabójstwo",
                "2 zabójstwa",
                "5 zabójstw",
                "11 zabójstw",
                "21 zabójstw",
                "22 zabójstwa",
                "25 zabójstw",
            ],
        ),
        (
            Locale::Lt,
            [
                "0 nužudymų",
                "1 nužudymas",
                "2 nužudymai",
                "5 nužudymai",
                "11 nužudymų",
                "21 nužudymas",
                "22 nužudymai",
                "25 nužudymai",
            ],
        ),
        (
            Locale::Ar,
            [
                "لا توجد عمليات قتل",
                "عملية قتل واحدة",
                "عمليتا قتل",
                "5 عمليات قتل",
                "11 عملية قتل",
                "21 عملية قتل",
                "22 عملية قتل",
                "25 عملية قتل",
            ],
        ),
    ];
    for (l, words) in expected {
        for (n, want) in counts.into_iter().zip(words) {
            assert_eq!(say(l, n), want, "{} {n}", l.tag());
        }
    }

    // Rules on the last digits keep working past 100, which the published
    // plural rules got wrong for both (vendor/README.md)
    for (l, n, want) in [
        (Locale::Lt, 111, "111 nužudymų"),
        (Locale::Lt, 122, "122 nužudymai"),
        (Locale::Lt, 101, "101 nužudymas"),
        (Locale::Ar, 103, "103 عمليات قتل"),
        (Locale::Ar, 111, "111 عملية قتل"),
        (Locale::Ar, 100, "100 عملية قتل"),
    ] {
        assert_eq!(say(l, n), want, "{} {n}", l.tag());
    }
}

#[test]
fn numbers_use_each_locale_separators() {
    let c = |l| Catalog::new(l).number(1234567.5, 2);
    assert_eq!(c(Locale::En), "1,234,567.50");
    assert_eq!(c(Locale::De), "1.234.567,50");
    assert_eq!(c(Locale::Ru), "1\u{a0}234\u{a0}567,50");
    assert_eq!(c(Locale::Fr), "1\u{202f}234\u{202f}567,50");
    assert_eq!(Catalog::new(Locale::Pl).number(1234., 0), "1234");
    assert_eq!(Catalog::new(Locale::Pl).number(12345., 0), "12\u{a0}345");
    assert_eq!(Catalog::new(Locale::En).number(-0.001, 1), "0.0");
    let score = |l| plain(Catalog::new(l).text_with("details-action", &[("score", 8.0603.into())]));
    assert_eq!(score(Locale::En), "Highest intensity 8.06");
    assert_eq!(score(Locale::De), "Höchste Intensität 8,06");
}

#[test]
fn system_languages_map_to_shipped_locales() {
    assert_eq!(Locale::from_tag("es-AR"), Some(Locale::Es419));
    assert_eq!(Locale::from_tag("es_ES"), Some(Locale::Es419));
    assert_eq!(Locale::from_tag("pt-PT"), Some(Locale::PtBr));
    assert_eq!(Locale::from_tag("zh-TW"), Some(Locale::ZhHans));
    assert_eq!(Locale::from_tag("uk-UA"), Some(Locale::Uk));
    assert_eq!(Locale::from_tag("nl-NL"), None);
}

#[test]
fn core_messages_fall_back_to_their_own_text() {
    let m = Message::new("test-unknown", "English text");
    assert_eq!(Catalog::new(Locale::De).message(&m), "English text");
}

#[test]
fn english_has_every_core_message_code() {
    let english = messages(Locale::En);
    for code in lambda_slicer_core::message::CODES {
        assert!(english.contains_key(*code), "en.ftl lacks {code}");
    }
}

#[test]
fn english_has_every_message_the_gui_names() {
    let english = messages(Locale::En);
    // Ids the code builds at run time, such as status-partial, are listed
    // here, the rest are found as literals in the source
    let built = [
        "status-complete",
        "status-partial",
        "status-failed",
        "status-unsupported",
        "kind-multiplayer-view",
        "kind-singleplayer",
        "kind-hltv",
        "kind-unknown",
        "mode-primary",
        "mode-charged",
        "mode-zoomed",
        "mode-unzoomed",
        "mode-grenade",
        "mode-bullets",
        "mode-single",
        "mode-double",
        "mode-rapid",
    ];
    let sources = [
        include_str!("../app.rs"),
        include_str!("../app/settings_window.rs"),
        include_str!("../library.rs"),
        include_str!("../jobs.rs"),
        include_str!("../demo.rs"),
        include_str!("../demo/compact.rs"),
        include_str!("../demo/events.rs"),
        include_str!("../demo/clips_tab.rs"),
        include_str!("../demo/scores.rs"),
    ];
    let mut named: Vec<String> = built.iter().map(|s| s.to_string()).collect();
    for source in sources {
        for literal in source.split('"').skip(1).step_by(2) {
            let is_id = literal.contains('-')
                && literal.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                && !literal.starts_with('-')
                && !literal.ends_with('-');
            if is_id {
                named.push(literal.to_string());
            }
        }
    }
    for id in named {
        assert!(english.contains_key(&id), "en.ftl lacks {id}");
    }
}

#[test]
fn pseudo_messages_grow_and_keep_their_arguments() {
    let mut c = Catalog::new(Locale::En);
    c.pseudo = true;
    assert_eq!(c.text("demo-close"), "[Çļöšé~~]");
    let t = plain(c.text_with("demo-loading", &[("name", "stalkx.dem".into())]));
    assert!(t.starts_with("[Åñàļýšîñĝ stalkx.dem…"), "{t}");
    assert!(t.ends_with("~~~~]"), "{t}");
}
