//! The library's counts and pages, read from the cache on a thread of their
//! own so a big library never holds up the window
//!
//! The view asks for every list's count when its filters, its order or the
//! stored files change, and for a page of rows as one scrolls into view.
//! Each ask carries the number of the count it belongs to. A count for new
//! filters stops the query under way, as whatever was asked before it is
//! out of date, and the asks of an earlier count are skipped. A query stopped
//! just as it began a newer count's work runs again. Every answer wakes
//! the window

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use eframe::egui;
use lambda_slicer_core::library::cache::{
    Cache, CacheError, ChatQuery, ChatRow, FileQuery, FileRow, Interrupter, LibraryStats,
    MatchQuery, MatchRow,
};
use lambda_slicer_core::message::{Coded, Message};

/// A query of one list's rows, one page long from its offset
pub enum PageOf {
    Demos(FileQuery),
    Matches(MatchQuery),
    Chat(ChatQuery),
}

/// A page of one list's rows
pub enum Rows {
    Demos(Vec<FileRow>),
    Matches(Vec<MatchRow>),
    Chat(Vec<ChatRow>),
}

/// How many rows each list has, the chat's only when asked for, and the
/// figures over the whole library
#[derive(Default)]
pub struct Counts {
    pub demos: u64,
    pub matches: u64,
    pub chat: Option<u64>,
    pub stats: LibraryStats,
}

/// What the thread is asked, the queries boxed as they are far bigger than
/// the rest
enum Ask {
    Count {
        number: u64,
        demos: Box<FileQuery>,
        matches: Box<MatchQuery>,
        chat: Option<Box<ChatQuery>>,
    },
    Page {
        number: u64,
        page: usize,
        of: Box<PageOf>,
    },
}

impl Ask {
    fn number(&self) -> u64 {
        match self {
            Ask::Count { number, .. } | Ask::Page { number, .. } => *number,
        }
    }
}

pub enum Answer {
    Counted {
        number: u64,
        counts: Result<Counts, Message>,
    },
    Page {
        number: u64,
        page: usize,
        rows: Result<Rows, Message>,
    },
}

/// The thread reading the cache at one path, and the way to it
pub struct Worker {
    asks: Sender<Ask>,
    answers: Receiver<Answer>,
    /// The newest count asked for, which the thread skips asks before
    newest: Arc<AtomicU64>,
    /// What stops the thread's query under way, once it opened the cache
    interrupter: Arc<Mutex<Option<Interrupter>>>,
}

impl Worker {
    /// A thread for the cache at `path`, which wakes `ctx` with each answer.
    /// It stops once the worker is dropped
    pub fn start(path: PathBuf, ctx: egui::Context) -> Self {
        let (asks, asked) = channel();
        let (answered, answers) = channel();
        let newest = Arc::new(AtomicU64::new(0));
        let interrupter = Arc::new(Mutex::new(None));

        let reader = Reader {
            path,
            cache: None,
            newest: newest.clone(),
            interrupter: interrupter.clone(),
        };
        let spawned = std::thread::Builder::new()
            .name("library".into())
            .spawn(move || reader.run(&asked, &answered, &ctx));
        if let Err(e) = spawned {
            eprintln!("cannot start the library's thread: {e}");
        }

        Self {
            asks,
            answers,
            newest,
            interrupter,
        }
    }

    /// Count every list again as count `number`, the chat's when `chat` is
    /// given, stopping whatever is under way when `stop` says so
    pub fn count(
        &self,
        number: u64,
        demos: FileQuery,
        matches: MatchQuery,
        chat: Option<ChatQuery>,
        stop: bool,
    ) {
        self.newest.store(number, Ordering::Relaxed);
        if stop
            && let Ok(slot) = self.interrupter.lock()
            && let Some(interrupter) = slot.as_ref()
        {
            interrupter.interrupt();
        }
        let ask = Ask::Count {
            number,
            demos: Box::new(demos),
            matches: Box::new(matches),
            chat: chat.map(Box::new),
        };
        let _ = self.asks.send(ask);
    }

    /// Read `page` of a list as count `number` has it
    pub fn page(&self, number: u64, page: usize, of: PageOf) {
        let of = Box::new(of);
        let _ = self.asks.send(Ask::Page { number, page, of });
    }

    /// The answers that came since last asked
    pub fn answers(&self) -> Vec<Answer> {
        self.answers.try_iter().collect()
    }
}

/// The thread's side, which opens the cache once it is there
struct Reader {
    path: PathBuf,
    cache: Option<Cache>,
    newest: Arc<AtomicU64>,
    interrupter: Arc<Mutex<Option<Interrupter>>>,
}

impl Reader {
    fn run(mut self, asked: &Receiver<Ask>, answered: &Sender<Answer>, ctx: &egui::Context) {
        while let Ok(ask) = asked.recv() {
            if ask.number() < self.newest.load(Ordering::Relaxed) {
                continue;
            }
            let answer = loop {
                match self.answer(&ask) {
                    Some(answer) => break Some(answer),
                    // Stopped for a newer count, or just as this one began
                    None if ask.number() < self.newest.load(Ordering::Relaxed) => break None,
                    None => {}
                }
            };
            let Some(answer) = answer else {
                continue;
            };
            if answered.send(answer).is_err() {
                return;
            }
            ctx.request_repaint();
        }
    }

    /// The cache, opened the first time it is there
    fn cache(&mut self) -> Result<Option<&Cache>, Message> {
        if self.cache.is_none() && self.path.exists() {
            let cache = Cache::open(&self.path).map_err(|e| e.message())?;
            if let Ok(mut slot) = self.interrupter.lock() {
                *slot = Some(cache.interrupter());
            }
            self.cache = Some(cache);
        }
        Ok(self.cache.as_ref())
    }

    /// The answer to `ask`, none when a query was stopped
    fn answer(&mut self, ask: &Ask) -> Option<Answer> {
        let cache = match self.cache() {
            Ok(cache) => cache,
            Err(e) => return Some(failed(ask, e)),
        };

        match ask {
            Ask::Count {
                number,
                demos,
                matches,
                chat,
            } => {
                let counts = match cache {
                    Some(cache) => counts(cache, demos, matches, chat.as_deref()),
                    None => Ok(Counts::default()),
                };
                let counts = settled(counts)?;
                let number = *number;
                Some(Answer::Counted { number, counts })
            }
            Ask::Page { number, page, of } => {
                let rows = match cache {
                    Some(cache) => rows(cache, of),
                    None => Ok(empty(of)),
                };
                let rows = settled(rows)?;
                let (number, page) = (*number, *page);
                Some(Answer::Page { number, page, rows })
            }
        }
    }
}

/// Every list's count, the chat's when asked for, and the library's
/// figures
fn counts(
    cache: &Cache,
    demos: &FileQuery,
    matches: &MatchQuery,
    chat: Option<&ChatQuery>,
) -> Result<Counts, CacheError> {
    let chat = match chat {
        Some(query) => Some(cache.count_chat(query)?),
        None => None,
    };
    Ok(Counts {
        demos: cache.count(demos)?,
        matches: cache.count_matches(matches)?,
        chat,
        stats: cache.library_stats()?,
    })
}

fn rows(cache: &Cache, of: &PageOf) -> Result<Rows, CacheError> {
    Ok(match of {
        PageOf::Demos(query) => Rows::Demos(cache.query(query)?),
        PageOf::Matches(query) => Rows::Matches(cache.query_matches(query)?),
        PageOf::Chat(query) => Rows::Chat(cache.query_chat(query)?),
    })
}

/// No rows of the list `of` asks for, for a library with no cache yet
fn empty(of: &PageOf) -> Rows {
    match of {
        PageOf::Demos(_) => Rows::Demos(Vec::new()),
        PageOf::Matches(_) => Rows::Matches(Vec::new()),
        PageOf::Chat(_) => Rows::Chat(Vec::new()),
    }
}

/// A query's result as an answer carries it, none when it was stopped
fn settled<T>(result: Result<T, CacheError>) -> Option<Result<T, Message>> {
    match result {
        Err(e) if e.interrupted() => None,
        Err(e) => Some(Err(e.message())),
        Ok(value) => Some(Ok(value)),
    }
}

/// The answer to `ask` when the cache could not be opened
fn failed(ask: &Ask, e: Message) -> Answer {
    match ask {
        Ask::Count { number, .. } => Answer::Counted {
            number: *number,
            counts: Err(e),
        },
        Ask::Page { number, page, .. } => Answer::Page {
            number: *number,
            page: *page,
            rows: Err(e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_slicer_core::library::cache::Entry;
    use std::path::Path;
    use std::time::{Duration, Instant};

    /// The worker's answers as they come, until `wanted` of them or `within`
    fn wait(worker: &Worker, wanted: usize, within: Duration) -> Vec<Answer> {
        let started = Instant::now();
        let mut out = Vec::new();
        while out.len() < wanted && started.elapsed() < within {
            out.extend(worker.answers());
            std::thread::sleep(Duration::from_millis(5));
        }
        out
    }

    #[test]
    fn counts_and_pages_come_back_and_outdated_asks_are_skipped() {
        let path = std::env::temp_dir().join(format!("ls-worker-{}.sqlite", std::process::id()));
        let mut cache = Cache::open(&path).unwrap();
        cache
            .store(&Entry {
                path: Path::new("a.dem"),
                size: 1,
                mtime_ns: None,
                file_identity: None,
                quick_hash: None,
                full_hash: None,
                header: None,
                report: None,
                error: None,
                floors: None,
            })
            .unwrap();

        let worker = Worker::start(path.clone(), egui::Context::default());
        let page = FileQuery {
            limit: 100,
            ..Default::default()
        };
        worker.count(1, FileQuery::default(), MatchQuery::default(), None, true);
        worker.page(1, 0, PageOf::Demos(page.clone()));
        let answers = wait(&worker, 2, Duration::from_secs(10));
        let Some(Answer::Counted { number: 1, counts }) = answers.first() else {
            panic!("no count");
        };
        assert_eq!(counts.as_ref().unwrap().demos, 1);
        assert!(counts.as_ref().unwrap().chat.is_none());
        let Some(Answer::Page { rows, .. }) = answers.get(1) else {
            panic!("no page");
        };
        assert!(matches!(rows, Ok(Rows::Demos(rows)) if rows.len() == 1));

        // A page of the count before a newer one is out of date, so the
        // newer count alone comes however long one waits
        worker.count(2, FileQuery::default(), MatchQuery::default(), None, true);
        worker.page(1, 0, PageOf::Demos(page));
        let answers = wait(&worker, 2, Duration::from_millis(500));
        assert_eq!(answers.len(), 1);
        assert!(matches!(answers[0], Answer::Counted { number: 2, .. }));

        drop(worker);
        drop(cache);
        for end in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{end}", path.display()));
        }
    }
}
