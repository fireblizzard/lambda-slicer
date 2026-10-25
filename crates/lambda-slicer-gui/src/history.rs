//! Where the user went, for mouse 4 and 5 to go back and forward through as
//! a web browser does
//!
//! The library comes first, then each demo or match opened, in order. Going
//! back and forward moves along them, and opening something new after going
//! back drops the places ahead, as a browser drops its forward pages

/// A place opened, and the moment its player was left at
#[derive(Clone, Debug, PartialEq)]
pub struct Visit<T> {
    pub place: T,
    pub at: Option<f32>,
}

#[derive(Debug)]
pub struct History<T> {
    /// The places after the library, in the order they were opened
    visits: Vec<Visit<T>>,
    /// How far along the user is, 0 at the library and 1 at the first visit
    now: usize,
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            visits: Vec::new(),
            now: 0,
        }
    }
}

impl<T> History<T> {
    /// The place shown, none at the library
    pub fn current(&self) -> Option<&Visit<T>> {
        let index = self.now.checked_sub(1)?;
        self.visits.get(index)
    }

    /// Note the moment the shown place is left at, for coming back to it
    pub fn leave(&mut self, at: Option<f32>) {
        let Some(index) = self.now.checked_sub(1) else {
            return;
        };
        if let Some(visit) = self.visits.get_mut(index)
            && at.is_some()
        {
            visit.at = at;
        }
    }

    /// Open `place` from where the user is, at `at`, dropping the places
    /// ahead
    pub fn visit(&mut self, place: T, at: Option<f32>) {
        self.visits.truncate(self.now);
        self.visits.push(Visit { place, at });
        self.now = self.visits.len();
    }

    /// Straight back to the library, keeping the places ahead
    pub fn home(&mut self) {
        self.now = 0;
    }

    /// One place back, false at the library
    pub fn back(&mut self) -> bool {
        if self.now == 0 {
            return false;
        }
        self.now -= 1;
        true
    }

    /// One place forward, false at the last
    pub fn forward(&mut self) -> bool {
        if self.now == self.visits.len() {
            return false;
        }
        self.now += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name of the place shown, "library" at the library
    fn shown(history: &History<&'static str>) -> &'static str {
        history.current().map_or("library", |v| v.place)
    }

    #[test]
    fn back_and_forward_go_through_every_place() {
        let mut history = History::default();
        history.visit("brazilian", None);
        history.visit("vengeance", None);
        history.visit("battlegrounds", None);

        assert!(history.back());
        assert_eq!(shown(&history), "vengeance");
        assert!(history.back());
        assert_eq!(shown(&history), "brazilian");
        assert!(history.forward());
        assert_eq!(shown(&history), "vengeance");
        assert!(history.back());
        assert!(history.back());
        assert_eq!(shown(&history), "library");
        assert!(!history.back());

        for _ in 0..3 {
            assert!(history.forward());
        }
        assert_eq!(shown(&history), "battlegrounds");
        assert!(!history.forward());
    }

    #[test]
    fn opening_after_going_back_drops_the_places_ahead() {
        let mut history = History::default();
        history.visit("brazilian", None);
        history.visit("vengeance", None);
        history.visit("battlegrounds", None);
        history.back();
        history.visit("crossfire", None);

        assert!(!history.forward());
        history.back();
        assert_eq!(shown(&history), "vengeance");
        history.back();
        assert_eq!(shown(&history), "brazilian");
    }

    #[test]
    fn home_keeps_the_places_ahead() {
        let mut history = History::default();
        history.visit("brazilian", None);
        history.visit("vengeance", None);
        history.home();
        assert_eq!(shown(&history), "library");

        history.forward();
        history.forward();
        assert_eq!(shown(&history), "vengeance");
    }

    #[test]
    fn a_place_comes_back_where_it_was_left() {
        let mut history = History::default();
        history.visit("brazilian", Some(10.));
        history.leave(Some(95.));
        history.visit("vengeance", None);
        history.back();
        assert_eq!(history.current().unwrap().at, Some(95.));

        // A player not open yet leaves the moment as it was
        history.leave(None);
        assert_eq!(history.current().unwrap().at, Some(95.));
    }
}
