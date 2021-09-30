use async_trait::async_trait;

#[allow(dead_code)]
mod app;
mod client;
mod components;
mod models;
mod option;

use std::sync::mpsc::Sender;
// use cloudwatchlogs::{Config, Credentials, Region};
// https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_StartQuery.html
use crate::{
    app::{view, App, Dispatcher, Message},
    client::{GroupsClient, QueryClient},
};
use client::{Client, SearchResult};

use crossterm::{
    event::{
        poll, read, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{
    error::Error,
    io::stdout,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use std::{fs::File, path::PathBuf};

use anyhow::{anyhow, Result};
use structopt::StructOpt;
use tui::{backend::CrosstermBackend, Terminal};

fn setup_logging() -> Result<()> {
    let mut path = get_app_cache_path()?;
    path.push("kanten.log");
    let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), File::create(path)?);
    Ok(())
}

fn get_app_cache_path() -> Result<PathBuf> {
    let mut path = dirs_next::cache_dir().ok_or_else(|| anyhow!("failed to find os cache dir."))?;
    path.push("kanten");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

#[derive(Debug, Clone)]
pub struct Messenger {
    pub tx: Sender<app::Message>,
}

impl Messenger {
    pub fn new(tx: Sender<app::Message>) -> Self {
        Self { tx }
    }
}

impl Dispatcher for Messenger {
    type Message = app::Message;

    fn dispatch(&self, message: app::Message) {
        log::debug!("Dispatched message is {:?}", message);
        self.tx.send(message).expect("failed to send message");
    }
}

#[async_trait]
pub trait AsyncTask {
    async fn run(&mut self, message: Message) -> Option<Message>;
}

struct Service {
    pub client: client::Client,
    // pub query_id: Option<crate::client::QueryId>,
}

#[async_trait]
impl AsyncTask for Service {
    async fn run(&mut self, message: Message) -> Option<Message> {
        match message {
            Message::GetQueryResultsRequest(query_id) => {
                log::trace!("request query result");
                if let Ok(SearchResult::Complete(items)) =
                    self.client.get_default_query_results(&query_id).await
                {
                    log::trace!("items {}", items.len());
                    Some(Message::GetQueryResultsComplete(items))
                } else {
                    // TODO: handle error
                    std::thread::sleep(Duration::from_millis(100));
                    self.run(Message::GetQueryResultsRequest(query_id)).await
                }
            }
            Message::StartQueryRequest(input) => {
                log::debug!("start query");
                let query_id = self
                    .client
                    .start_default_query(input.clone())
                    .await
                    .expect(&format!("Failed to start query {:?}", input));
                Some(Message::StartQueryComplete(query_id))
            }
            Message::StopQueryRequest(query_id) => {
                let _ = self.client.stop_query(&query_id).await;
                None
            }
            _ => Some(message),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logging()?;

    let opt = option::Opt::from_args();

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(cloudwatchlogs::Client::new(&shared_config));
    let group_names = client.get_group_names().await?;

    let (tx0, rx0) = mpsc::channel::<Message>();
    let (tx1, rx1) = mpsc::channel::<Message>();

    enable_raw_mode()?;

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    let tx2 = tx0.clone();
    // Setup input handling
    let tick_rate = Duration::from_millis(50);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if poll(timeout).unwrap() {
                if let CEvent::Key(key) = read().unwrap() {
                    tx2.send(Message::KeyInput(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx2.send(Message::Tick).expect("Failed to send tick event");
                last_tick = Instant::now();
            }
        }
    });

    let messenger = Messenger::new(tx1);
    let mut task = Service {
        // query_id: None,
        client: client.clone(),
    };

    tokio::spawn(async move {
        loop {
            let r = rx1.recv();
            if let Ok(r) = r {
                if let Some(m) = task.run(r).await {
                    let _result = tx0.send(m);
                }
            }
        }
    });

    let mut app = App::new(messenger, group_names.items, opt);
    terminal.clear()?;

    loop {
        terminal.draw(|f| view::draw(f, &mut app))?;
        let message = rx0.recv()?;
        match message {
            Message::KeyInput(key) => match key {
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                _ => app.on_key(key)?,
            },
            // TODO: remove await
            _ => app.update(message).await,
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
