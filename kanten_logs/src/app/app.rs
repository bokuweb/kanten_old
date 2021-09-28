use crossterm::event::{KeyCode, KeyEvent};

use anyhow::Result;

use tui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};

use crossterm::event::KeyModifiers;

use crate::{client::*, components::*};
use crate::{models::Duration, option::Opt};

#[derive(Debug, PartialEq)]
pub enum FocusTarget {
    LogFilter,
    Duration,
    GroupFilter,
    Groups,
    Logs,
    FindStringInLogs,
}

pub struct App<'a, D>
where
    D: Dispatcher,
{
    pub loading: bool,
    pub dispatcher: D,
    pub focus_state: FocusTarget,
    pub should_quit: bool,
    pub should_query_restart: bool,
    pub group_names: GroupList,
    pub logs: LogListModel,
    pub duration: Duration,
    pub query_started: bool,
    pub query_completed: bool,
    pub default_query_input: InputModel<'a>,
    pub group_filter_input: InputModel<'a>,
    pub find_string_input: InputModel<'a>,
    pub duration_input: InputModel<'a>,
    pub query_id: Option<QueryId>,
}

pub trait Dispatcher {
    type Message;

    fn dispatch(&self, message: Self::Message);
}

#[derive(Debug)]
pub enum Message {
    Tick,
    KeyInput(KeyEvent),
    GetQueryResultsRequest(QueryId),
    GetQueryResultsComplete(Vec<SearchResultItem>),
    StartQueryRequest(StartQueryInput),
    StartQueryComplete(QueryId),
    StopQueryRequest(QueryId),
}

impl<'a, D: Dispatcher<Message = Message>> App<'a, D> {
    pub fn new(dispatcher: D, group_names: Vec<String>, opt: Opt) -> App<'a, D> {
        let mut default_query_input = InputModel::new().set_placeholder("Filter your logs");
        default_query_input.focus();

        let group_name_filter = opt.group_name.unwrap_or_default();
        let group_filter_input = InputModel::new()
            .set_placeholder("Filter log groups")
            .set_value(group_name_filter.clone())
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default()),
            );
        let group_names = GroupList::with_items(group_names, group_name_filter, true);

        let find_string_input = InputModel::new()
            .set_placeholder("Find string in logs")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .focused_block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            );

        let duration = Duration::from_opt(&opt.since, opt.end.as_deref());
        let duration_input_value = if opt.end.is_none() {
            opt.since
        } else {
            format!("{} - {}", opt.since, opt.end.unwrap())
        };

        let duration_input = InputModel::new()
            .set_placeholder("duration(default 15m)")
            .set_value(duration_input_value)
            .block(
                Block::default()
                    .borders(Borders::LEFT)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .focused_block(
                Block::default()
                    .borders(Borders::LEFT)
                    .border_style(Style::default()),
            );

        App {
            loading: false,
            dispatcher,
            focus_state: FocusTarget::LogFilter,
            should_quit: false,
            should_query_restart: false,
            group_names,
            logs: LogListModel::new(),
            duration,
            query_id: None,
            query_started: false,
            query_completed: false,
            default_query_input,
            duration_input,
            group_filter_input,
            find_string_input,
        }
    }

    pub fn on_key(&mut self, k: KeyEvent) -> Result<()> {
        match k {
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
            } => self.focus_next(),
            KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::CONTROL,
            } => self.focus_prev(),
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            } => match self.focus_state {
                FocusTarget::LogFilter => {
                    self.should_query_restart = true;
                    self.request_stop_query();
                    self.logs.clear();
                }
                FocusTarget::Duration => {
                    let duration: Duration = self.duration_input.value().into();
                    if duration.is_valid() {
                        // TODO: error handling
                        self.duration = duration;
                        self.should_query_restart = true;
                        self.request_stop_query();
                        self.logs.clear();
                    }
                }
                FocusTarget::Groups => {
                    self.group_names.on_key(k);
                    self.should_query_restart = true;
                    self.request_stop_query();
                    self.logs.clear();
                }
                _ => {}
            },
            _ => match self.focus_state {
                FocusTarget::LogFilter => self.default_query_input.on_key(k),
                FocusTarget::Duration => self.duration_input.on_key(k),
                FocusTarget::GroupFilter => {
                    self.group_filter_input.on_key(k);
                    self.group_names.set_filter(self.group_filter_input.value());
                }
                FocusTarget::Logs => self.logs.on_key(k),
                FocusTarget::Groups => self.group_names.on_key(k),
                FocusTarget::FindStringInLogs => {
                    self.find_string_input.on_key(k);
                    self.logs.set_find_text(self.find_string_input.value());
                }
                _ => {}
            },
        }
        Ok(())
    }

    pub fn request_stop_query(&mut self) {
        if let Some(ref id) = self.query_id {
            log::trace!("stop query");
            self.dispatcher
                .dispatch(Message::StopQueryRequest(id.clone()));
            self.query_id = None;
        }
    }

    pub async fn on_tick(&mut self) {
        if !self.query_started || self.should_query_restart {
            log::trace!("restart query");
            self.should_query_restart = false;

            let groups: Vec<String> = self.group_names.selected.clone().into_iter().collect();
            if !groups.is_empty() && self.duration.is_valid() {
                let Duration::Duration { start, end } = self.duration;
                self.query_started = true;
                self.loading = true;
                self.dispatcher
                    .dispatch(Message::StartQueryRequest(StartQueryInput {
                        start: start.unwrap(),
                        end: end.unwrap(),
                        filter: self.default_query_input.value().to_string(),
                        groups,
                    }));
            }
        }
    }

    fn blur_all(&mut self) {
        self.default_query_input.blur();
        self.duration_input.blur();
        self.group_filter_input.blur();
        self.find_string_input.blur();
    }

    pub fn focus_next(&mut self) {
        self.blur_all();
        match self.focus_state {
            FocusTarget::LogFilter => {
                self.duration_input.focus();
                self.focus_state = FocusTarget::Duration;
            }
            FocusTarget::Duration => {
                self.group_filter_input.focus();
                self.focus_state = FocusTarget::GroupFilter;
            }
            FocusTarget::GroupFilter => {
                self.focus_state = FocusTarget::Groups;
            }
            FocusTarget::Groups => self.focus_state = FocusTarget::Logs,
            FocusTarget::Logs => {
                self.find_string_input.focus();
                self.focus_state = FocusTarget::FindStringInLogs;
            }
            FocusTarget::FindStringInLogs => {
                self.default_query_input.focus();
                self.focus_state = FocusTarget::LogFilter;
            }
        }
    }

    pub fn focus_prev(&mut self) {
        self.blur_all();
        match self.focus_state {
            FocusTarget::LogFilter => {
                self.find_string_input.focus();
                self.focus_state = FocusTarget::FindStringInLogs;
            }
            FocusTarget::Duration => {
                self.default_query_input.focus();
                self.focus_state = FocusTarget::LogFilter;
            }
            FocusTarget::GroupFilter => {
                self.duration_input.focus();
                self.focus_state = FocusTarget::Duration;
            }
            FocusTarget::Groups => {
                self.group_filter_input.focus();
                self.focus_state = FocusTarget::GroupFilter;
            }
            FocusTarget::Logs => {
                self.focus_state = FocusTarget::Groups;
            }
            FocusTarget::FindStringInLogs => {
                self.focus_state = FocusTarget::Logs;
            }
        }
    }

    pub async fn update(&mut self, message: Message) {
        log::trace!("update message {:?}", message);
        match message {
            Message::GetQueryResultsComplete(items) => {
                for item in items {
                    self.logs.push(LogListItem::new(item.message))
                }
                self.query_completed = true;
                self.loading = false;
                self.query_id = None;
            }
            Message::StartQueryComplete(query_id) => {
                log::trace!("StartQueryComplete");
                self.query_id = Some(query_id.clone());
                self.query_started = true;
                self.dispatcher
                    .dispatch(Message::GetQueryResultsRequest(query_id));
                self.query_completed = false;
            }
            Message::Tick => {
                self.on_tick().await;
            }
            _ => {}
        }
    }
}
