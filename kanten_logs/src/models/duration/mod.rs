use chrono;
use humantime::{parse_duration, parse_rfc3339};

#[derive(Debug)]
pub enum Duration {
    // Live,
    Duration {
        start: Option<i64>,
        end: Option<i64>,
    },
}

fn parse(s: &str) -> Option<i64> {
    let now = chrono::Local::now();
    if s == "now" {
        return Some(now.timestamp());
    }

    if let Ok(d) = parse_duration(s) {
        Some((now - chrono::Duration::seconds(d.as_secs() as i64)).timestamp())
    } else {
        if let Ok(d) = parse_rfc3339(s) {
            if let Ok(d) = d.elapsed() {
                return Some(d.as_secs() as i64);
            }
        }
        None
    }
}

impl Duration {
    pub fn from_opt(s: &str, e: Option<&str>) -> Self {
        let end = if let Some(e) = e {
            parse(e)
        } else {
            parse("now")
        };
        Self::Duration {
            start: parse(s),
            end,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Duration::Duration { start, end } => start.is_some() && end.is_some(),
            // _ => todo!(),
        }
    }
}

impl From<&str> for Duration {
    fn from(s: &str) -> Self {
        let s: Vec<Option<i64>> = s.split('-').map(|s| parse(s.trim())).collect();
        let start: Option<i64> = s.get(0).and_then(|i| *i);
        let end: Option<i64> = s.get(1).and_then(|i| *i);
        Duration::Duration { start, end }
    }
}
