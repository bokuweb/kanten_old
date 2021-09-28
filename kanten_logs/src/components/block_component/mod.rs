use tui::{backend::Backend, layout::Rect, Frame};

pub trait BlockComponent {
    fn draw<B: Backend>(self, f: &mut Frame<B>, area: Rect);
}
