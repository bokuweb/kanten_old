use crate::app::{app::FocusTarget, App, Dispatcher, Message};
use crate::components::*;

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw<B, D: Dispatcher<Message = Message>>(f: &mut Frame<B>, app: &mut App<D>)
where
    B: Backend,
{
    let vertical = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .direction(Direction::Vertical)
        .split(f.size());

    draw_query_form(f, app, vertical[0]);

    let horizontal = Layout::default()
        .constraints([Constraint::Min(8), Constraint::Length(1)].as_ref())
        .split(vertical[1]);
    draw_body(f, app, horizontal[0]);
    draw_status(f, app, horizontal[1]);
}

fn draw_query_form<B, D: Dispatcher<Message = Message>>(
    f: &mut Frame<B>,
    app: &mut App<D>,
    area: Rect,
) where
    B: Backend,
{
    let border_color = if app.default_query_input.is_focused() || app.duration_input.is_focused() {
        Color::White
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Log filter")
        .border_style(Style::default().fg(border_color));

    f.render_widget(block, area);

    let inner_chunks = Layout::default()
        .vertical_margin(1)
        .horizontal_margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(30)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    let query_input = InputView::new(&app.default_query_input);
    query_input.draw(f, inner_chunks[0]);

    let duration_input = InputView::new(&app.duration_input);
    duration_input.draw(f, inner_chunks[1]);
}

fn draw_body<B, D: Dispatcher<Message = Message>>(f: &mut Frame<B>, app: &mut App<D>, area: Rect)
where
    B: Backend,
{
    let constraints = vec![Constraint::Percentage(100)];
    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(chunks[0]);
        {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[0]);

            draw_groups(f, app, chunks[0]);
            draw_logs(f, app, chunks[1]);
        }
    }
}

fn draw_status<B, D: Dispatcher<Message = Message>>(f: &mut Frame<B>, app: &mut App<D>, area: Rect)
where
    B: Backend,
{
    let text = if app.loading {
        vec![Spans::from("loading...")]
    } else if app.too_much_groups_specified {
        vec![Spans::from("too much groups specified. uncheck some groups...")]
    } else {
        vec![Spans::from(format!(
            "{} items found.",
            app.logs.items.len()
        ))]
    };
    let block = Block::default()
        .style(Style::default().bg(Color::Rgb(72, 68, 96)))
        .borders(Borders::NONE);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_groups<B, D: Dispatcher<Message = Message>>(f: &mut Frame<B>, app: &mut App<D>, area: Rect)
where
    B: Backend,
{
    // Draw groups
    let filter = regex::escape(&app.group_names.filter);
    let re = regex::Regex::new(&filter).expect("Failed to construct Regexp");
    let groups: Vec<ListItem> = app
        .group_names
        .items
        .iter()
        .filter(|name| re.is_match(name))
        .map(|item| {
            let style = Style::default();
            let mut line = Checkbox::from(app.group_names.selected.contains(item)).render();
            line.0.extend(vec![Span::raw(" "), Span::raw(item)]);
            ListItem::new(line).style(style)
        })
        .collect();

    let border_color =
        if app.focus_state == FocusTarget::Groups || app.focus_state == FocusTarget::GroupFilter {
            Color::White
        } else {
            Color::DarkGray
        };

    f.render_widget(
        Block::default()
            .title("Groups")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
        area,
    );

    let inner_chunks = Layout::default()
        .vertical_margin(1)
        .horizontal_margin(1)
        .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
        .direction(Direction::Vertical)
        .split(area);

    let input = InputView::new(&app.group_filter_input);
    input.draw(f, inner_chunks[0]);

    let groups = List::new(groups)
        .highlight_style(if app.focus_state == FocusTarget::Groups {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(72, 68, 96))
        } else {
            Style::default()
        })
        .highlight_symbol(if app.focus_state == FocusTarget::Groups {
            "▸"
        } else {
            " "
        });

    f.render_stateful_widget(groups, inner_chunks[1], &mut app.group_names.state);
}

fn draw_logs<B, D: Dispatcher<Message = Message>>(f: &mut Frame<B>, app: &mut App<D>, area: Rect)
where
    B: Backend,
{
    log::debug!("{} items", app.logs.items.len());

    let inner_chunks = Layout::default()
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .direction(Direction::Vertical)
        .split(area);

    let input = InputView::new(&app.find_string_input);
    input.draw(f, inner_chunks[1]);

    let border_color = if app.focus_state == FocusTarget::Logs {
        Color::White
    } else {
        Color::DarkGray
    };

    let log_block = Block::default()
        .borders(Borders::ALL)
        .title("Logs")
        .border_style(Style::default().fg(border_color));

    if app.loading {
        let text = vec![Spans::from("loading...")];
        let paragraph = Paragraph::new(text)
            .block(log_block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    if app.logs.items.is_empty() {
        let text = vec![Spans::from("No items")];
        let paragraph = Paragraph::new(text)
            .block(log_block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let logs = LogList::new(&app.logs.items)
        .block(log_block)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White)
                .bg(Color::Rgb(72, 68, 96)),
        );
    f.render_stateful_widget(logs, inner_chunks[0], &mut app.logs.state);
}
