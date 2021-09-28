use tui::{
    style::{Color, Style},
    text::{Span, Spans},
};

use super::inline_component::*;

pub struct Checkbox {
    pub checked: bool,
}

impl Checkbox {
    pub fn from(checked: bool) -> Checkbox {
        Self { checked }
    }
}

impl InlineComponent for Checkbox {
    fn render<'a>(&self) -> Spans<'a> {
        if self.checked {
            Spans::from(vec![Span::styled(
                "◉",
                Style::default().fg(Color::Rgb(43, 116, 100)),
            )])
        } else {
            Spans::from(vec![Span::raw("◯")])
        }
    }
}
