use std::collections::BTreeSet;
use tui::widgets::ListState;
use crate::app;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct GroupList {
    pub state: ListState,
    pub items: Vec<String>,
    pub selected: BTreeSet<String>,
    pub filter: String,
    pub filtered: Vec<String>,
}

impl GroupList {
    pub fn with_items(
        items: Vec<String>,
        filter: impl Into<String>,
        default_select: bool,
    ) -> GroupList {
        let mut state = ListState::default();
        let filter = filter.into();
        let filtered: Vec<String> = if filter.is_empty() {
            items.clone()
        } else {
            let re = regex::Regex::new(&filter).expect("Failed to construct Regexp");
            items
                .iter()
                .filter(|name| re.is_match(name))
                .cloned()
                .collect()
        };
        let selected: std::collections::BTreeSet<String> = if !filter.is_empty() && default_select {
            filtered
                .iter()
                .cloned()
                .take(app::SPECIFIABLE_GROUPS_COUNT)
                .collect()
        } else {
            BTreeSet::new()
        };
        state.select(Some(0));

        GroupList {
            state,
            items,
            selected,
            filtered,
            filter,
        }
    }

    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        if !self.filter.is_empty() {
            let re = regex::Regex::new(&self.filter).expect("Failed to construct Regexp");
            self.filtered = self
                .items
                .iter()
                .filter(|name| re.is_match(name))
                .cloned()
                .collect()
        } else {
            self.filtered = self.items.clone();
        };
    }

    // pub fn clear(&mut self) {
    //     // self.state.offset = 0;
    //     self.state.select(Some(0));
    // }

    // pub fn to_list_items(&self) -> Vec<ListItem> {
    //     let re = regex::Regex::new(&self.filter).expect("Failed to construct Regexp");
    //
    //     self.items
    //         .iter()
    //         .filter(|name| re.is_match(name))
    //         .map(|item| {
    //             let style = Style::default();
    //             let mut line = Checkbox::from(self.selected.contains(item)).render();
    //             line.0.extend(vec![Span::raw(" "), Span::raw(item)]);
    //             ListItem::new(line).style(style)
    //         })
    //         .collect()
    // }

    pub fn on_key(&mut self, key: KeyEvent) {
        match key {
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            } => {
                log::debug!("on group_list key {:?}", self.state.selected());
                if let Some(index) = self.state.selected() {
                    let name = &self.filtered[index];
                    log::debug!("name {:?}", name);
                    if self.selected.contains(name) {
                        self.selected.remove(name);
                    } else {
                        self.selected.insert(name.clone());
                    }
                }
            }
            // down
            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
            } => self.next(),
            // up
            KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
            } => self.previous(),
            _ => {}
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    // pub fn unselect(&mut self) {
    //     self.state.select(None);
    // }
}
