mod line_builder;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{Dispatcher, Message};

use self::line_builder::LineBuilder;

pub struct LogListModel<D: Dispatcher<Message = Message>> {
    pub state: LogListState<D>,
    pub items: Vec<LogListItem>,
}

#[derive(Debug, Clone)]
pub struct LogListState<D: Dispatcher<Message = Message>> {
    offset: usize,
    selected: Option<usize>,
    focused: bool,
    find_text: String,
    end_index: usize,
    prev_page_start_index: usize,
    dispatcher: D,
}

impl<D: Dispatcher<Message = Message>> LogListState<D> {
    pub fn new(dispatcher: D) -> LogListState<D> {
        LogListState {
            offset: 0,
            selected: None,
            focused: false,
            prev_page_start_index: 0,
            end_index: 0,
            find_text: String::default(),
            dispatcher,
        }
    }
}

impl<D: Dispatcher<Message = Message>> LogListState<D> {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }
}

impl<D: Dispatcher<Message = Message>> LogListModel<D> {
    pub fn new(d: D) -> Self {
        let mut state = LogListState::new(d);
        state.select(Some(0));
        LogListModel {
            state,
            items: Vec::new(),
        }
    }

    pub fn set_find_text(&mut self, t: impl Into<String>) {
        self.state.find_text = t.into();
    }

    pub fn push(&mut self, item: LogListItem) {
        self.items.push(item);
    }

    pub fn clear(&mut self) {
        self.items = vec![];
        self.state.offset = 0;
        self.state.selected = Some(0);
    }

    pub fn next_if_exist(&mut self) {
        if let Some(i) = self.state.selected() {
            if i < self.items.len() - 1 {
                self.state.select(Some(i + 1));
            }
        };
    }

    pub fn previous_if_exist(&mut self) {
        if let Some(i) = self.state.selected() {
            if i > 0 {
                self.state.select(Some(i - 1));
            }
        };
    }

    pub fn next_page_if_exist(&mut self) {
        if self.state.end_index < self.items.len() - 1 {
            self.state.select(Some(self.state.end_index));
            self.state.offset = self.state.end_index;
        }
    }

    pub fn previous_page_if_exist(&mut self) {
        self.state.select(Some(self.state.prev_page_start_index));
        self.state.offset = self.state.prev_page_start_index;
    }

    // pub fn unselect(&mut self) {
    //     self.state.select(None);
    // }
    //
    // pub fn focus(&mut self) {
    //     self.state.focused = true;
    // }
    //
    // pub fn blur(&mut self) {
    //     self.state.focused = false;
    // }

    pub fn update_end_index(&mut self, index: usize) {
        self.state.end_index = index;
    }

    pub fn update_prev_page_start_index(&mut self, index: usize) {
        self.state.prev_page_start_index = index;
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match key {
            // down
            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
            } => self.next_if_exist(),
            // up
            KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
            } => self.previous_if_exist(),
            // page up
            KeyEvent {
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::ALT,
            }
            | KeyEvent {
                code: KeyCode::PageUp,
                modifiers: KeyModifiers::NONE,
            } => self.previous_page_if_exist(),
            // page down
            KeyEvent {
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::PageDown,
                modifiers: KeyModifiers::NONE,
            } => self.next_page_if_exist(),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct LogListItem {
    log: String,
    timestamp: String,
    style: Style,
    line_builder: LineBuilder,
}

impl LogListItem {
    pub fn new(timestamp: String, log: String) -> Self {
        LogListItem {
            log,
            timestamp,
            style: Style::default(),
            line_builder: LineBuilder::new(),
        }
    }

    // pub fn style(mut self, style: Style) -> Self {
    //     self.style = style;
    //     self
    // }

    pub fn text(&self) -> String {
        format!("{}    {}", self.timestamp, self.log)
    }

    pub fn height(&self, w: u16) -> usize {
        self.line_builder.run_composer(&self.text(), w, "").len()
    }
}

#[derive(Debug)]
pub struct LogList<'a, D> {
    block: Option<Block<'a>>,
    items: &'a [LogListItem],
    style: Style,
    highlight_style: Style,
    _phantom: std::marker::PhantomData<fn() -> D>,
}

impl<'a, D> LogList<'a, D> {
    pub fn new(items: &'a [LogListItem]) -> LogList<'a, D> {
        Self {
            block: None,
            style: Style::default(),
            items,
            highlight_style: Style::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> LogList<'a, D> {
        self.block = Some(block);
        self
    }

    // pub fn style(mut self, style: Style) -> LogList<'a> {
    //     self.style = style;
    //     self
    // }

    pub fn highlight_style(mut self, style: Style) -> LogList<'a, D> {
        self.highlight_style = style;
        self
    }
}

impl<'a, D: Dispatcher<Message = Message>> StatefulWidget for LogList<'a, D> {
    type State = LogListState<D>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.set_style(area, self.style);
        let list_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if list_area.width < 1 || list_area.height < 1 {
            return;
        }

        if self.items.is_empty() {
            return;
        }
        let list_height = list_area.height as usize;

        let mut start = state.offset;
        let mut end = state.offset;

        let mut height = 0;

        for item in self.items.iter().skip(state.offset) {
            let item_height = item.height(list_area.width);
            if height + item_height > list_height {
                if height != list_height {
                    let overflow = (height + item_height - list_height) as u16;
                    height = height + item_height - overflow as usize;
                    end += 1;
                }
                break;
            }
            end += 1;
            height += item_height;
        }

        let selected = state.selected.unwrap_or(0).min(self.items.len() - 1);
        while selected >= end {
            height = height.saturating_add(self.items[end].height(list_area.width));
            end += 1;
            while height > list_height {
                height = height.saturating_sub(self.items[start].height(list_area.width));
                start += 1;
            }
        }
        while selected < start {
            start -= 1;
            height = height.saturating_add(self.items[start].height(list_area.width));
            while height > list_height {
                end -= 1;
                height = height.saturating_sub(self.items[end].height(list_area.width));
            }
        }
        state.offset = start;
        let mut current_height = 0;
        for (i, item) in self
            .items
            .iter()
            .enumerate()
            .skip(state.offset)
            .take(end - start)
        {
            let item_height = item.height(list_area.width) as u16;
            let (x, y) = {
                let pos = (list_area.left(), list_area.top() + current_height);
                current_height += item_height as u16;
                pos
            };

            if y >= list_area.bottom() {
                break;
            }

            let area = Rect {
                x,
                y,
                width: list_area.width,
                height: (item_height as u16).wrapping_sub(
                    if list_area.bottom() > y as u16 + item_height as u16 {
                        0
                    } else {
                        (y as u16 + item_height as u16).wrapping_sub(list_area.bottom())
                    },
                ),
            };
            let item_style = self.style.patch(item.style);
            buf.set_style(area, item_style);

            let is_selected = state.selected.map(|s| s == i).unwrap_or(false);
            let elem_x = x;

            if is_selected {
                buf.set_style(area, self.highlight_style);
            }

            let max_element_width = (list_area.width - (elem_x - x)) as usize;
            for (j, line) in item
                .line_builder
                .run_composer(&item.text(), list_area.width, &state.find_text)
                .iter()
                .enumerate()
            {
                if y + (j as u16) < list_area.bottom() {
                    buf.set_spans(
                        elem_x,
                        y + j as u16,
                        // pan::raw(line),,
                        line,
                        max_element_width as u16,
                    );
                }
            }
        }

        if state.end_index != end {
            state
                .dispatcher
                .dispatch(crate::app::Message::UpdateLogListEndIndex(end));
        }

        let mut prev_page_start_index = state.offset;
        let mut height = 0;
        loop {
            if prev_page_start_index == 0 {
                break;
            }
            let item_height = self.items[prev_page_start_index].height(list_area.width);
            height += item_height;
            if height > list_height {
                break;
            }
            prev_page_start_index -= 1;
        }

        if state.prev_page_start_index != prev_page_start_index {
            state
                .dispatcher
                .dispatch(crate::app::Message::UpdateLogListPrevPageStartIndex(
                    prev_page_start_index,
                ));
        }
    }
}
