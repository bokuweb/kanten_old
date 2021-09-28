use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Widget},
    Frame,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use unicode_segmentation::UnicodeSegmentation;

use super::BlockComponent;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    x: u16,
    y: u16,
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputModel<'a> {
    style: Style,
    value: String,
    placeholder: String,
    cursor_position: Position,
    focused: bool,
    block: Option<Block<'a>>,
    focused_block: Option<Block<'a>>,
}

impl<'a> InputModel<'a> {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            value: String::default(),
            placeholder: String::default(),
            cursor_position: Position::default(),
            focused: false,
            block: None,
            focused_block: None,
        }
    }

    pub fn set_value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    pub fn set_placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn clamp_by(&self, w: usize) -> String {
        UnicodeSegmentation::graphemes(self.value.as_str(), true)
            .take(w)
            .collect::<Vec<&str>>()
            .join("")
    }

    pub fn placeholder(&self) -> &str {
        self.placeholder.as_str()
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn focused_block(mut self, focused_block: Block<'a>) -> Self {
        self.focused_block = Some(focused_block);
        self
    }

    #[allow(dead_code)]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn blur(&mut self) {
        self.focused = false;
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match key {
            // start
            KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                self.cursor_position.x = 0;
            }
            // end
            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                let size = UnicodeSegmentation::graphemes(self.value(), true).count() as u16;
                self.cursor_position.x = size;
            }
            // ->
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
            } => {
                let size = UnicodeSegmentation::graphemes(self.value(), true).count() as u16;
                if self.cursor_position.x == size {
                    return;
                }
                self.cursor_position.x += 1;
            }
            // <-
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
            } => self.cursor_position.x = self.cursor_position.x.saturating_sub(1),
            // input
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
            } => {
                self.value.insert(self.cursor_position.x as usize, c);
                self.cursor_position.x += 1;
            }
            // delete
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Delete,
                modifiers: KeyModifiers::NONE,
            } => {
                if self.value.is_empty() {
                    return;
                }
                self.value.remove(self.cursor_position.x as usize);
            }
            // backspace
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::CONTROL,
            }
            | KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
            } => {
                if self.cursor_position.x == 0 {
                    return;
                }
                self.value.remove(self.cursor_position.x as usize - 1);
                self.cursor_position.x -= 1;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputView<'a> {
    model: &'a InputModel<'a>,
}

impl<'a> InputView<'a> {
    pub fn new(model: &'a InputModel) -> Self {
        Self { model }
    }
}

impl<'a> BlockComponent for InputView<'a> {
    fn draw<B: Backend>(self, f: &mut Frame<B>, area: Rect) {
        let cursor_position = self.model.cursor_position;
        let focused = self.model.focused;
        let inner_area = match self.model.block.as_ref() {
            Some(b) => b.inner(area),
            None => area,
        };
        f.render_widget(self, area);
        if focused && (inner_area.left() + cursor_position.x) < inner_area.right() {
            f.set_cursor(inner_area.left() + cursor_position.x, inner_area.top());
        }
    }
}

impl<'a> Widget for InputView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.model.style);
        let block = if self.model.is_focused() {
            if self.model.focused_block.is_some() {
                self.model.focused_block.clone()
            } else {
                self.model.block.clone()
            }
        } else {
            self.model.block.clone()
        };
        let area = match block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let (top, _height) = {
            let area = area.inner(&Margin {
                horizontal: 0,
                vertical: 0,
            });

            (area.top(), area.height)
        };

        if self.model.value().is_empty() {
            buf.set_string(
                area.left(),
                top,
                self.model.placeholder(),
                Style::default().fg(Color::DarkGray),
            );
        } else {
            buf.set_string(
                area.left(),
                top,
                self.model.clamp_by((area.right() - area.left()) as usize),
                Style::default(),
            );
        }
    }
}
