use std::cell::RefCell;

use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct LineBuilder {
    pub lines_cache: RefCell<lru::LruCache<String, Vec<Spans<'static>>>>,
}

impl LineBuilder {
    pub fn new() -> Self {
        Self {
            lines_cache: RefCell::new(lru::LruCache::new(2)),
        }
    }

    pub fn width(&self, text: &str) -> usize {
        text.width()
    }

    pub fn run_composer<'a>(
        &self,
        text: &'a str,
        text_area_width: u16,
        find_text: &str,
    ) -> Vec<Spans<'a>> {
        let key = format!("{}-{}-{}", text.to_owned(), text_area_width, find_text);
        if let Some(c) = self.lines_cache.borrow_mut().get(&key) {
            return c.clone();
        }
        let find_text = regex::escape(find_text);
        let mut lines: Vec<Spans> = vec![];

        for t in text.split('\n') {
            if t.is_empty() {
                continue;
            }
            let re =
                regex::Regex::new(&format!("(?i)({})", find_text)).expect("Failed to build regex");
            let mut caps = re.find_iter(t);

            if self.width(text) as u16 <= text_area_width {
                let mut spans = vec![];
                for t in re.split(text) {
                    spans.push(Span::raw(t.to_owned()));
                    if let Some(c) = caps.next() {
                        spans.push(Span::styled(
                            c.as_str().to_string(),
                            Style::default()
                                .bg(Color::Rgb(238, 173, 15))
                                .fg(Color::Black),
                        ));
                    }
                }
                lines.push(Spans::from(spans));
                continue;
            }
            let mut line: Vec<Span> = vec![];
            let mut line_text = String::new();
            let mut line_width: u16 = 0;
            let mut cap = if find_text.is_empty() {
                None
            } else {
                caps.next()
            };

            for (i, t) in UnicodeSegmentation::graphemes(text, true).enumerate() {
                let w = self.width(t) as u16;

                if let Some(c) = cap {
                    if i == c.start() && !line_text.is_empty() {
                        line.push(Span::styled(line_text.clone(), Style::default()));
                        line_text = "".to_owned();
                    }

                    if i == c.end() && !line_text.is_empty() {
                        line.push(Span::styled(
                            line_text.clone(),
                            Style::default()
                                .bg(Color::Rgb(238, 173, 15))
                                .fg(Color::Black),
                        ));
                        line_text = "".to_owned();
                    }

                    if i == c.end() {
                        cap = caps.next();
                    }
                }

                if line_width + w > text_area_width {
                    if !line_text.is_empty() {
                        if let Some(c) = cap {
                            if i >= c.start() && i <= c.end() {
                                line.push(Span::styled(
                                    line_text.clone(),
                                    Style::default()
                                        .bg(Color::Rgb(238, 173, 15))
                                        .fg(Color::Black),
                                ));
                            } else {
                                line.push(Span::styled(line_text.clone(), Style::default()));
                            }
                        } else {
                            line.push(Span::styled(line_text.clone(), Style::default()));
                        }
                    }
                    lines.push(Spans::from(line));
                    line = vec![];
                    line_text = t.to_string();
                    line_width = w;
                } else {
                    line_text += t;
                    line_width += w;
                }
            }

            if !line_text.is_empty() {
                line.push(Span::styled(line_text, Style::default()));
            }

            if !line.is_empty() {
                lines.push(Spans::from(line));
            }
        }
        self.lines_cache.borrow_mut().put(key, lines.clone());
        lines
    }
}
