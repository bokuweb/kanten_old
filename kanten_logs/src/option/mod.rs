use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kanten")]
pub struct Opt {
    /// Return logs newer than a relative duration like 52, 2m, or 3h. (default: "15m")
    #[structopt(short, long, default_value = "15m")]
    pub since: String,

    /// Return logs older than a relative duration like 0, 2m, or 3h.
    #[structopt(short, long)]
    pub end: Option<String>,

    /// Log group name (regular expression).
    #[structopt(short, long)]
    pub group_name: Option<String>,
}
