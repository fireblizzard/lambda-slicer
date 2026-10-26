//! Sentences, the speech the game puts together from words
//!
//! `sound/sentences.txt` names a sentence on each line, such as
//! `HG_GREN1 hgrunt/(t30) clik take!(e75) cover!(s5) clik`. A line that
//! starts with `/` is a comment. A sentence is found by its name, ignoring
//! case, or by its number, its place among the lines that are not comments.
//! Its words are files of the folder before the last `/`, or of `vox/` when
//! it names none, and `,` and `.` are the words `_comma` and `_period`.
//! Parentheses right after a word change that word, and parentheses on their
//! own change every word after them. In them `v` is the volume, `p` the
//! pitch, `s` and `e` where the word starts and ends and `t` how much of it
//! is skipped, each in percent
//!
//! The words are joined into one sound with no gap between them, and the
//! quiet at either end of each word is trimmed as the engine does. A word's
//! pitch speeds it up by its percent, where the engine adds its difference
//! to the sound's own pitch, which is the same at the usual pitch. Skipping
//! drops that share of every short piece of the word

use lambda_slicer_assets::wav::Sound;

/// The most sentences the engine reads
const MOST_SENTENCES: usize = 4096;

/// How far into each end of a word quiet is trimmed, in samples, and how
/// loud a sample must be not to count as quiet
const TRIM_SCAN: usize = 255;
const TRIM_BELOW: f32 = 512.;

/// How long the pieces skipping drops a share of are, and how long their
/// joins fade into each other, in seconds
const PIECE: f64 = 0.02;
const FADE: f64 = 0.002;

/// The sentences of `sentences.txt`, in its order
#[derive(Debug, Default)]
pub struct Sentences {
    /// Each sentence's name and its words
    lines: Vec<(String, String)>,
}

impl Sentences {
    pub fn parse(text: &str) -> Self {
        let mut lines = Vec::new();
        for line in text.lines() {
            let line = line.trim_start();
            if line.is_empty() || line.starts_with('/') {
                continue;
            }
            if lines.len() == MOST_SENTENCES {
                break;
            }

            let (name, words) = line.split_once([' ', '\t']).unwrap_or((line, ""));
            lines.push((name.to_owned(), words.trim().to_owned()));
        }
        Self { lines }
    }

    /// The words of the sentence `name`, `!` left out, which is its number
    /// when it is all digits
    pub fn find(&self, name: &str) -> Option<&str> {
        let numbered = name.parse::<usize>().ok().and_then(|n| self.lines.get(n));
        let found = numbered.or_else(|| {
            let named = |(line, _): &&(String, String)| line.eq_ignore_ascii_case(name);
            self.lines.iter().find(named)
        });
        found.map(|(_, words)| words.as_str())
    }
}

/// How a word is played, each in percent
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordParams {
    pub volume: u32,
    pub pitch: u32,
    /// Where the word starts and ends, played to its end when the end is
    /// not after the start
    pub start: u32,
    pub end: u32,
    /// How much of the word is skipped
    pub skip: u32,
}

impl Default for WordParams {
    fn default() -> Self {
        Self {
            volume: 100,
            pitch: 100,
            start: 0,
            end: 100,
            skip: 0,
        }
    }
}

impl WordParams {
    /// These with the changes inside parentheses, such as `t30 p105`
    fn with(mut self, changes: &str) -> Self {
        let mut letter = None;
        let mut number = String::new();
        // A space at the end sets the last change
        for c in changes.chars().chain([' ']) {
            if c.is_ascii_digit() {
                number.push(c);
                continue;
            }

            if let (Some(letter), Ok(value)) = (letter, number.parse::<u32>()) {
                self.set(letter, value);
            }
            letter = Some(c.to_ascii_lowercase());
            number.clear();
        }
        self
    }

    fn set(&mut self, letter: char, value: u32) {
        let percent = value.min(100);
        match letter {
            'v' => self.volume = percent,
            'p' => self.pitch = value.clamp(1, 255),
            's' => self.start = percent,
            'e' => self.end = percent,
            't' => self.skip = percent,
            _ => {}
        }
    }
}

/// A word of a sentence, its file relative to `sound/`
#[derive(Debug, PartialEq, Eq)]
pub struct Word {
    pub file: String,
    pub params: WordParams,
}

/// The words of a sentence as `sentences.txt` writes them
pub fn words(sentence: &str) -> Vec<Word> {
    let sentence = sentence.trim();
    let sentence = sentence.strip_prefix('/').unwrap_or(sentence);
    let (folder, sentence) = match sentence.rfind('/') {
        Some(at) => sentence.split_at(at + 1),
        None => ("vox/", sentence),
    };

    let mut words = Vec::new();
    let mut defaults = WordParams::default();
    let mut word = String::new();
    let add = |words: &mut Vec<Word>, word: &mut String, params| {
        if !word.is_empty() {
            let file = format!("{folder}{word}.wav");
            words.push(Word { file, params });
            word.clear();
        }
    };

    let mut chars = sentence.chars();
    while let Some(c) = chars.next() {
        match c {
            '(' => {
                let mut changes = String::new();
                for c in chars.by_ref() {
                    if c == ')' {
                        break;
                    }
                    changes.push(c);
                }

                // On their own they change every word after them
                if word.is_empty() {
                    defaults = defaults.with(&changes);
                } else {
                    add(&mut words, &mut word, defaults.with(&changes));
                }
            }
            ',' | '.' => {
                add(&mut words, &mut word, defaults);
                let mut mark = String::from(if c == ',' { "_comma" } else { "_period" });
                add(&mut words, &mut mark, defaults);
            }
            c if c.is_whitespace() => add(&mut words, &mut word, defaults),
            c => word.push(c),
        }
    }
    add(&mut words, &mut word, defaults);
    words
}

/// The words one after another as one sound, at the highest rate among
/// them and in mono. None when there are no words
pub fn join(words: &[(&Sound, WordParams)]) -> Option<Sound> {
    let rate = words.iter().map(|(sound, _)| sound.rate).max()?;

    let mut samples = Vec::new();
    for &(sound, params) in words {
        let word = trimmed(sound, params);
        let word = skipped(word, sound.rate, params.skip);
        let speed = f64::from(sound.rate) * f64::from(params.pitch) / 100. / f64::from(rate);
        let volume = params.volume as f32 / 100.;
        for sample in resampled(&word, speed) {
            samples.push((sample * volume).clamp(-32768., 32767.) as i16);
        }
    }

    let frames = u32::try_from(samples.len()).ok()?;
    Some(Sound {
        rate,
        channels: 1,
        samples,
        loop_start: None,
        frames,
    })
}

/// The part of the word `params` play, in mono, without the quiet at its
/// ends
fn trimmed(sound: &Sound, params: WordParams) -> Vec<f32> {
    let channels = usize::from(sound.channels.max(1));
    let frames = (sound.frames as usize).min(sound.samples.len() / channels);
    let mono: Vec<f32> = sound.samples[..frames * channels]
        .chunks(channels)
        .map(|frame| frame.iter().map(|&s| f32::from(s)).sum::<f32>() / channels as f32)
        .collect();

    let at = |percent: u32| frames * percent as usize / 100;
    let mut start = at(params.start);
    let mut end = if params.end > params.start {
        at(params.end)
    } else {
        frames
    };

    let quiet = |sample: f32| sample.abs() < TRIM_BELOW;
    let first = start;
    while start < end && start - first < TRIM_SCAN && quiet(mono[start]) {
        start += 1;
    }
    let last = end;
    while end > start && last - end < TRIM_SCAN && quiet(mono[end - 1]) {
        end -= 1;
    }
    mono[start..end].to_vec()
}

/// The word with `skip` percent of each of its pieces dropped, at `rate`
fn skipped(word: Vec<f32>, rate: u32, skip: u32) -> Vec<f32> {
    if skip == 0 {
        return word;
    }

    let piece = ((f64::from(rate) * PIECE) as usize).max(1);
    let fade = (f64::from(rate) * FADE) as usize;
    let kept = piece * (100 - skip as usize) / 100;
    let mut joined: Vec<f32> = Vec::with_capacity(word.len());
    for part in word.chunks(piece) {
        // Each piece but the first fades in over the end of the one before,
        // so it keeps that much more
        let fade = fade.min(joined.len());
        let part = &part[..(kept + fade).min(part.len())];
        let fade = fade.min(part.len());
        let overlap = joined.len() - fade;
        for (i, &sample) in part[..fade].iter().enumerate() {
            let t = (i + 1) as f32 / (fade + 1) as f32;
            let before = &mut joined[overlap + i];
            *before = *before * (1. - t) + sample * t;
        }
        joined.extend_from_slice(&part[fade..]);
    }
    joined
}

/// The word read `speed` samples a sample, between samples drawn straight
fn resampled(word: &[f32], speed: f64) -> Vec<f32> {
    if word.is_empty() {
        return Vec::new();
    }

    let length = (word.len() as f64 / speed).floor() as usize;
    let mut out = Vec::with_capacity(length);
    for i in 0..length {
        let at = i as f64 * speed;
        let before = at.floor() as usize;
        let after = (before + 1).min(word.len() - 1);
        let t = (at - before as f64) as f32;
        out.push(word[before] * (1. - t) + word[after] * t);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(file: &str, params: WordParams) -> Word {
        Word {
            file: file.into(),
            params,
        }
    }

    #[test]
    fn sentences_are_found_by_name_or_number_past_comments() {
        let s = Sentences::parse(
            "// HUMAN GRUNTS\n\nHG_GREN0 hgrunt/clik\r\n  HG_GREN1\thgrunt/(t30) clik\n",
        );
        assert_eq!(s.find("hg_gren1"), Some("hgrunt/(t30) clik"));
        assert_eq!(s.find("0"), Some("hgrunt/clik"));
        assert_eq!(s.find("1"), Some("hgrunt/(t30) clik"));
        assert_eq!(s.find("2"), None);
        assert_eq!(s.find("HG_ALERT0"), None);
    }

    #[test]
    fn parentheses_change_their_word_or_every_word_after_them() {
        let compressed = WordParams {
            skip: 30,
            ..WordParams::default()
        };
        assert_eq!(
            words("hgrunt/(t30) clik take!(e75) cover!(s5) clik"),
            vec![
                word("hgrunt/clik.wav", compressed),
                word(
                    "hgrunt/take!.wav",
                    WordParams {
                        end: 75,
                        ..compressed
                    }
                ),
                word(
                    "hgrunt/cover!.wav",
                    WordParams {
                        start: 5,
                        ..compressed
                    }
                ),
                word("hgrunt/clik.wav", compressed),
            ]
        );
    }

    #[test]
    fn commas_and_periods_are_words_and_vox_is_the_folder_by_default() {
        let normal = WordParams::default();
        assert_eq!(
            words("freeman!(t20 p105), clik(p110v80)."),
            vec![
                word(
                    "vox/freeman!.wav",
                    WordParams {
                        skip: 20,
                        pitch: 105,
                        ..normal
                    }
                ),
                word("vox/_comma.wav", normal),
                word(
                    "vox/clik.wav",
                    WordParams {
                        pitch: 110,
                        volume: 80,
                        ..normal
                    }
                ),
                word("vox/_period.wav", normal),
            ]
        );
    }

    fn sound(rate: u32, samples: Vec<i16>) -> Sound {
        let frames = samples.len() as u32;
        Sound {
            rate,
            channels: 1,
            samples,
            loop_start: None,
            frames,
        }
    }

    #[test]
    fn words_join_trimmed_at_the_highest_rate() {
        let quiet_ends = sound(11025, vec![0, 10, 1000, 2000, 1000, 5, 0]);
        let fast = sound(22050, vec![3000, 3000]);
        let normal = WordParams::default();
        let joined = join(&[(&quiet_ends, normal), (&fast, normal)]).unwrap();
        assert_eq!(joined.rate, 22050);
        // The slower word takes twice the samples at the higher rate
        assert_eq!(joined.frames, 2 * 3 + 2);
        assert_eq!(joined.samples[0], 1000);
        assert_eq!(joined.samples[6..], [3000, 3000]);
    }

    #[test]
    fn a_word_plays_from_its_start_to_its_end_at_its_volume() {
        let ramp = sound(100, (1..=10).map(|i| i * 1000).collect());
        let params = WordParams {
            start: 20,
            end: 50,
            volume: 50,
            ..WordParams::default()
        };
        let joined = join(&[(&ramp, params)]).unwrap();
        assert_eq!(joined.samples, [1500, 2000, 2500]);
    }

    #[test]
    fn skipping_shortens_a_word_by_its_share() {
        let long = sound(1000, vec![4000; 1000]);
        let params = WordParams {
            skip: 25,
            ..WordParams::default()
        };
        let joined = join(&[(&long, params)]).unwrap();
        // A little more, as the pieces' joins fade into each other
        assert!((750..=752).contains(&joined.frames));
        assert!(joined.samples.iter().all(|&s| s == 4000));
    }
}
