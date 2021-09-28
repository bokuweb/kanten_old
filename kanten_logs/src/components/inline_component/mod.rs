use tui::text::Spans;

pub trait InlineComponent {
    fn render<'a>(&self) -> Spans<'a>;
}
