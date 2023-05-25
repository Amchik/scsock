use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config<A = RawAction> {
    pub socket: String,
    #[serde(default, rename = "remove-socket-if-exists")]
    pub remove_socket_if_exists: bool,

    pub actions: IndexMap<String, A>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum RawAction {
    Plain(String),
    Verbose {
        #[serde(rename = "do")]
        command: String,
        #[serde(default)]
        name: Option<String>,
    },
}

pub struct Action {
    pub name: String,
    pub command: String,
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!("`Action` should never be deserialized. Use `RawAction`")
    }
}

impl Config {
    pub fn parse(s: &str) -> Config {
        let r: Result<Config, _> = toml::from_str(s);
        match r {
            Ok(r) => r,
            Err(e) => {
                eprintln!("\u{1b}[1;31merror\u{1b}[0;1m: {}\u{1b}[0m", e.message());
                if let Some(r) = e.span() {
                    eprintln!("   \u{1b}[34m:=\u{1b}[0m range: {}..{}", r.start, r.end);
                    for l in s[r].lines() {
                        eprintln!("   \u{1b}[34m|\u{1b}[0m {l}");
                    }
                }
                std::process::exit(1)
            }
        }
    }

    pub fn get_action_title(&self, v: usize) -> &str {
        match &self
            .actions
            .get_index(v)
            .expect("invalid index in `get_action_title()`")
        {
            (_, RawAction::Verbose { name: Some(v), .. }) => v.as_str(),
            (v, _) => v.as_str(),
        }
    }
    pub fn get_action_command(&self, v: usize) -> &str {
        match &self
            .actions
            .get_index(v)
            .expect("invalid index in `get_action_command()`")
            .1
        {
            RawAction::Plain(command) | RawAction::Verbose { command, .. } => command.as_str(),
        }
    }
}
