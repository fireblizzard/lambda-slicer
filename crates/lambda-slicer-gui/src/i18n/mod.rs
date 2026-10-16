//! Interface languages (implementation plan §8.1)
//!
//! One Fluent catalog per locale is embedded in the binary. A message
//! missing from a locale falls back to English, and a core message missing
//! from every catalog falls back to its own English text. Numbers use the
//! locale's separators, with the digits 0 to 9 in every locale

use fluent_bundle::types::FluentNumber;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use lambda_slicer_core::library::cache::StoredMessage;
use lambda_slicer_core::message::{Arg, Message};
use unic_langid::LanguageIdentifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Locale {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "es-419")]
    Es419,
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "ru")]
    Ru,
    #[serde(rename = "uk")]
    Uk,
    #[serde(rename = "tr")]
    Tr,
    #[serde(rename = "pl")]
    Pl,
    #[serde(rename = "de")]
    De,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "it")]
    It,
    #[serde(rename = "fi")]
    Fi,
    #[serde(rename = "lt")]
    Lt,
    #[serde(rename = "ar")]
    Ar,
    #[serde(rename = "zh-Hans")]
    ZhHans,
}

/// Decimal and group separators, and the shortest number that is grouped
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberStyle {
    pub decimal: char,
    pub group: char,
    pub min_grouped: u64,
}

const POINT_COMMA: NumberStyle = NumberStyle {
    decimal: '.',
    group: ',',
    min_grouped: 1000,
};
const COMMA_POINT: NumberStyle = NumberStyle {
    decimal: ',',
    group: '.',
    min_grouped: 1000,
};
const COMMA_SPACE: NumberStyle = NumberStyle {
    decimal: ',',
    group: '\u{a0}',
    min_grouped: 1000,
};
/// Polish groups only from five digits
const COMMA_SPACE_5: NumberStyle = NumberStyle {
    decimal: ',',
    group: '\u{a0}',
    min_grouped: 10_000,
};
/// French groups with a narrow no-break space
const COMMA_NARROW: NumberStyle = NumberStyle {
    decimal: ',',
    group: '\u{202f}',
    min_grouped: 1000,
};

impl Locale {
    pub const ALL: [Locale; 14] = [
        Locale::En,
        Locale::Es419,
        Locale::PtBr,
        Locale::Ru,
        Locale::Uk,
        Locale::Tr,
        Locale::Pl,
        Locale::De,
        Locale::Fr,
        Locale::It,
        Locale::Fi,
        Locale::Lt,
        Locale::Ar,
        Locale::ZhHans,
    ];

    /// BCP 47 tag
    pub fn tag(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Es419 => "es-419",
            Locale::PtBr => "pt-BR",
            Locale::Ru => "ru",
            Locale::Uk => "uk",
            Locale::Tr => "tr",
            Locale::Pl => "pl",
            Locale::De => "de",
            Locale::Fr => "fr",
            Locale::It => "it",
            Locale::Fi => "fi",
            Locale::Lt => "lt",
            Locale::Ar => "ar",
            Locale::ZhHans => "zh-Hans",
        }
    }

    /// The language's name in itself, as the language menu shows it
    pub fn native_name(self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Es419 => "Español (Latinoamérica)",
            Locale::PtBr => "Português (Brasil)",
            Locale::Ru => "Русский",
            Locale::Uk => "Українська",
            Locale::Tr => "Türkçe",
            Locale::Pl => "Polski",
            Locale::De => "Deutsch",
            Locale::Fr => "Français",
            Locale::It => "Italiano",
            Locale::Fi => "Suomi",
            Locale::Lt => "Lietuvių",
            Locale::Ar => "العربية",
            Locale::ZhHans => "简体中文",
        }
    }

    pub fn rtl(self) -> bool {
        self == Locale::Ar
    }

    /// Whether a native speaker has reviewed the catalog
    pub fn reviewed(self) -> bool {
        self == Locale::En
    }

    pub fn number_style(self) -> NumberStyle {
        match self {
            Locale::En | Locale::Es419 | Locale::Ar | Locale::ZhHans => POINT_COMMA,
            Locale::PtBr | Locale::Tr | Locale::De | Locale::It => COMMA_POINT,
            Locale::Ru | Locale::Uk | Locale::Fi | Locale::Lt => COMMA_SPACE,
            Locale::Pl => COMMA_SPACE_5,
            Locale::Fr => COMMA_NARROW,
        }
    }

    fn source(self) -> &'static str {
        match self {
            Locale::En => include_str!("ftl/en.ftl"),
            Locale::Es419 => include_str!("ftl/es-419.ftl"),
            Locale::PtBr => include_str!("ftl/pt-BR.ftl"),
            Locale::Ru => include_str!("ftl/ru.ftl"),
            Locale::Uk => include_str!("ftl/uk.ftl"),
            Locale::Tr => include_str!("ftl/tr.ftl"),
            Locale::Pl => include_str!("ftl/pl.ftl"),
            Locale::De => include_str!("ftl/de.ftl"),
            Locale::Fr => include_str!("ftl/fr.ftl"),
            Locale::It => include_str!("ftl/it.ftl"),
            Locale::Fi => include_str!("ftl/fi.ftl"),
            Locale::Lt => include_str!("ftl/lt.ftl"),
            Locale::Ar => include_str!("ftl/ar.ftl"),
            Locale::ZhHans => include_str!("ftl/zh-Hans.ftl"),
        }
    }

    /// The locale for a system language tag. Any Spanish maps to Latin
    /// American Spanish, any Portuguese to Brazilian and any Chinese to
    /// Simplified, the only variants shipped
    pub fn from_tag(tag: &str) -> Option<Locale> {
        let tag = tag.replace('_', "-");
        let language = tag.split('-').next()?.to_ascii_lowercase();
        Some(match language.as_str() {
            "en" => Locale::En,
            "es" => Locale::Es419,
            "pt" => Locale::PtBr,
            "ru" => Locale::Ru,
            "uk" => Locale::Uk,
            "tr" => Locale::Tr,
            "pl" => Locale::Pl,
            "de" => Locale::De,
            "fr" => Locale::Fr,
            "it" => Locale::It,
            "fi" => Locale::Fi,
            "lt" => Locale::Lt,
            "ar" => Locale::Ar,
            "zh" => Locale::ZhHans,
            _ => return None,
        })
    }

    /// The first of the system's preferred languages that is shipped, or
    /// English
    pub fn detect() -> Locale {
        sys_locale::get_locales()
            .find_map(|t| Locale::from_tag(&t))
            .unwrap_or(Locale::En)
    }
}

/// `value` with `fraction` decimals in the locale's style
pub fn format_number(style: NumberStyle, value: f64, fraction: usize) -> String {
    let text = format!("{:.*}", fraction, value.abs());
    let (int, frac) = text.split_once('.').unwrap_or((&text, ""));
    let mut grouped = String::new();
    let digits: Vec<char> = int.chars().collect();
    let group = value.abs() >= style.min_grouped as f64;
    for (i, d) in digits.iter().enumerate() {
        if group && i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(style.group);
        }
        grouped.push(*d);
    }
    let sign = if value < 0. && text.chars().any(|c| c != '0' && c != '.') {
        "-"
    } else {
        ""
    };
    if frac.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped}{}{frac}", style.decimal)
    }
}

/// Fluent number formatter for one style. Fluent takes a plain function, so
/// each style has its own instance
fn formatter<const S: usize>(
    value: &FluentValue,
    _: &intl_memoizer::IntlLangMemoizer,
) -> Option<String> {
    let FluentValue::Number(FluentNumber { value, options }) = value else {
        return None;
    };
    let style = [
        POINT_COMMA,
        COMMA_POINT,
        COMMA_SPACE,
        COMMA_SPACE_5,
        COMMA_NARROW,
    ][S];
    let fraction = options
        .minimum_fraction_digits
        .or(options.maximum_fraction_digits)
        .unwrap_or(if value.fract() == 0. { 0 } else { 2 });

    Some(format_number(style, *value, fraction))
}

fn bundle(locale: Locale) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = locale.tag().parse().expect("valid tag");
    let mut bundle = FluentBundle::new(vec![id]);
    let resource = FluentResource::try_new(locale.source().to_string())
        .unwrap_or_else(|(_, errors)| panic!("{} catalog: {errors:?}", locale.tag()));
    bundle
        .add_resource(resource)
        .unwrap_or_else(|errors| panic!("{} catalog: {errors:?}", locale.tag()));
    bundle.add_builtins().expect("builtins");
    let style = locale.number_style();
    let f = if style == POINT_COMMA {
        formatter::<0>
    } else if style == COMMA_POINT {
        formatter::<1>
    } else if style == COMMA_SPACE {
        formatter::<2>
    } else if style == COMMA_SPACE_5 {
        formatter::<3>
    } else {
        formatter::<4>
    };
    bundle.set_formatter(Some(f));
    bundle
}

fn fluent_args(args: &[(&str, FluentValue<'static>)]) -> FluentArgs<'static> {
    let mut out = FluentArgs::new();
    for (name, value) in args {
        out.set(name.to_string(), value.clone());
    }
    out
}

/// A loaded catalog with English behind it
pub struct Catalog {
    pub locale: Locale,
    bundle: FluentBundle<FluentResource>,
    english: FluentBundle<FluentResource>,
}

impl Catalog {
    pub fn new(locale: Locale) -> Self {
        Self {
            locale,
            bundle: bundle(locale),
            english: bundle(Locale::En),
        }
    }

    fn format(
        bundle: &FluentBundle<FluentResource>,
        id: &str,
        args: Option<&FluentArgs>,
    ) -> Option<String> {
        let pattern = bundle.get_message(id)?.value()?;
        let mut errors = Vec::new();
        let text = bundle.format_pattern(pattern, args, &mut errors);
        errors.is_empty().then(|| text.into_owned())
    }

    /// The message `id` with `args`, from this locale or else English. An id
    /// that English lacks is a bug, shown as the id itself
    pub fn text_with(&self, id: &str, args: &[(&str, FluentValue<'static>)]) -> String {
        let args = (!args.is_empty()).then(|| fluent_args(args));
        Self::format(&self.bundle, id, args.as_ref())
            .or_else(|| Self::format(&self.english, id, args.as_ref()))
            .unwrap_or_else(|| id.to_string())
    }

    pub fn text(&self, id: &str) -> String {
        self.text_with(id, &[])
    }

    /// A core message in this locale, or its own English text when no
    /// catalog has its code
    pub fn message(&self, m: &Message) -> String {
        let mut args = FluentArgs::new();
        for (name, value) in &m.args {
            let value = match value {
                Arg::Integer(i) => FluentValue::from(*i),
                Arg::Number(n) => FluentValue::from(*n),
                Arg::Text(t) => FluentValue::from(t.clone()),
            };
            args.set(*name, value);
        }
        Self::format(&self.bundle, m.code, Some(&args))
            .or_else(|| Self::format(&self.english, m.code, Some(&args)))
            .unwrap_or_else(|| m.text.clone())
    }

    /// A message read back from the library cache, as [`Self::message`]
    pub fn stored(&self, m: &StoredMessage) -> String {
        let mut args = FluentArgs::new();
        if let serde_json::Value::Object(map) = &m.args {
            for (name, value) in map {
                let value = match value {
                    serde_json::Value::Number(n) => n
                        .as_i64()
                        .map(FluentValue::from)
                        .or_else(|| n.as_f64().map(FluentValue::from))
                        .unwrap_or(FluentValue::None),
                    serde_json::Value::String(t) => FluentValue::from(t.clone()),
                    other => FluentValue::from(other.to_string()),
                };
                args.set(name.clone(), value);
            }
        }
        Self::format(&self.bundle, &m.code, Some(&args))
            .or_else(|| Self::format(&self.english, &m.code, Some(&args)))
            .unwrap_or_else(|| m.text.clone())
    }

    /// A number in this locale's style
    pub fn number(&self, value: f64, fraction: usize) -> String {
        format_number(self.locale.number_style(), value, fraction)
    }
}

#[cfg(test)]
mod tests;
