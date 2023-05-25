use std::{
    fs,
    io::Write,
    os::unix::net::{UnixListener, UnixStream},
    process::Command,
    thread,
};

use cfg::{Config, RawAction};
use clap::{Parser, Subcommand};
use msg::Message;

mod cfg;
mod msg;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Override config socket
    #[arg(short, long)]
    socket: Option<String>,

    /// Path to configuration file
    #[arg(short, long, default_value = "scsock.toml")]
    config: String,

    #[command(subcommand)]
    subcommands: Subcommands,
}
#[derive(Subcommand)]
enum Subcommands {
    /// List existing commands
    List,
    /// Get current command
    Get,
    /// Send a command
    Send {
        /// ID of command to send
        id: u8,
    },
    /// Send a next command
    Next,
    /// Start a server
    Start,
}

fn main() {
    let Args {
        socket,
        config,
        subcommands,
    } = Args::parse();
    let config = match fs::read_to_string(&config) {
        Ok(v) => Config::parse(&v),
        Err(e) => {
            eprintln!("\u{1b}[1;31merror\u{1b}[0;1m: failed to read file {config}\u{1b}[0m: {e}");
            std::process::exit(1)
        }
    };
    let mut id = 0;
    let socket = socket.unwrap_or_else(|| config.socket.clone());

    match subcommands {
        Subcommands::List => {
            for (i, (id, act)) in config.actions.iter().enumerate() {
                let (title, cmd) = match act {
                    RawAction::Verbose {
                        command,
                        name: Some(name),
                    } => (name.as_str(), command),
                    RawAction::Plain(command) | RawAction::Verbose { command, .. } => {
                        (id.as_str(), command)
                    }
                };
                println!("\u{1b}[2m{i}\u{1b}[0;1m: {title}\u{1b}[0m ({cmd})");
            }
        }
        Subcommands::Start => {
            // cry about it
            if config.remove_socket_if_exists {
                _ = fs::remove_file(&socket);
            }
            let socket = match UnixListener::bind(&socket) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "\u{1b}[1;31merror\u{1b}[0;1m: failed to bind to {socket}\u{1b}[0m: {e}"
                    );
                    std::process::exit(1)
                }
            };
            for conn in socket.incoming() {
                match conn {
                    Ok(mut s) => {
                        let msg = match Message::read(&mut s) {
                            Ok(Some(m)) => m,
                            Ok(None) => {
                                if let Err(e) = s.write_all(&Message::ReErrUnkwn.as_raw_bytes()) {
                                    eprintln!(
                                    "\u{1b}[1;33mwarning\u{1b}[0;1m: failed to write incoming\u{1b}[0m: {e}"
                                );
                                }
                                break;
                            }
                            Err(e) => {
                                eprintln!(
                                    "\u{1b}[1;33mwarning\u{1b}[0;1m: failed to read incoming\u{1b}[0m: {e}"
                                );
                                continue;
                            }
                        };
                        let r = match msg {
                            Message::GetStatus => {
                                let title = config.get_action_title(id).as_bytes();
                                s.write_all(&Message::ReStatus(title.to_owned()).as_raw_bytes())
                            }
                            Message::SetID(i) if i as usize >= config.actions.len() => {
                                s.write_all(&Message::ReErrNoID.as_raw_bytes())
                            }
                            Message::SetID(i) => {
                                id = i as usize;
                                let title = config.get_action_title(id).as_bytes();
                                let cmd = config.get_action_command(id);
                                let r = Command::new("/bin/sh").arg("-c").arg(cmd).spawn();
                                match r {
                                    Ok(mut r) => {
                                        thread::spawn(move || {
                                            // do not spawn zombies:
                                            _ = r.wait();
                                        });
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "\u{1b}[1;33mwarning\u{1b}[0;1m: failed to spawn command\u{1b}[0m: {e}"
                                        );
                                    }
                                };
                                s.write_all(&Message::ReStatus(title.to_owned()).as_raw_bytes())
                            }
                            Message::NextID => {
                                id = if id + 1 < config.actions.len() {
                                    id + 1
                                } else {
                                    0
                                };
                                let title = config.get_action_title(id).as_bytes();
                                let cmd = config.get_action_command(id);
                                let r = Command::new("/bin/sh").arg("-c").arg(cmd).spawn();
                                match r {
                                    Ok(mut r) => {
                                        thread::spawn(move || {
                                            // do not spawn zombies:
                                            _ = r.wait();
                                        });
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "\u{1b}[1;33mwarning\u{1b}[0;1m: failed to spawn command\u{1b}[0m: {e}"
                                        );
                                    }
                                };
                                s.write_all(&Message::ReStatus(title.to_owned()).as_raw_bytes())
                            }
                            _ => s.write_all(&Message::ReErrIdiot.as_raw_bytes()),
                        };
                        if let Err(e) = r {
                            eprintln!(
                                "\u{1b}[1;33mwarning\u{1b}[0;1m: failed to write incoming\u{1b}[0m: {e}"
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "\u{1b}[1;31merror\u{1b}[0;1m: failed to connect to incoming\u{1b}[0m: {e}"
                        );
                        std::process::exit(1)
                    }
                }
            }
        }
        Subcommands::Get | Subcommands::Send { .. } | Subcommands::Next => {
            let mut stream = match UnixStream::connect(&socket) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "\u{1b}[1;31merror\u{1b}[0;1m: failed to connect to {socket}\u{1b}[0m: {e}"
                    );
                    std::process::exit(1)
                }
            };
            match subcommands {
                Subcommands::Get => {
                    if let Err(e) = stream.write_all(&Message::GetStatus.as_raw_bytes()) {
                        eprintln!(
                            "\u{1b}[1;31merror\u{1b}[0;1m: failed to write to socket\u{1b}[0m: {e}"
                        );
                        std::process::exit(1)
                    }
                }
                Subcommands::Send { id } => {
                    if let Err(e) = stream.write_all(&Message::SetID(id).as_raw_bytes()) {
                        eprintln!(
                            "\u{1b}[1;31merror\u{1b}[0;1m: failed to write to socket\u{1b}[0m: {e}"
                        );
                        std::process::exit(1)
                    }
                }
                Subcommands::Next => {
                    if let Err(e) = stream.write_all(&Message::NextID.as_raw_bytes()) {
                        eprintln!(
                            "\u{1b}[1;31merror\u{1b}[0;1m: failed to write to socket\u{1b}[0m: {e}"
                        );
                        std::process::exit(1)
                    }
                }
                _ => unreachable!(),
            };
            let msg = match Message::read(&mut stream) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "\u{1b}[1;31merror\u{1b}[0;1m: failed to write to socket\u{1b}[0m: {e}"
                    );
                    std::process::exit(1)
                }
            };
            match msg {
                Some(Message::ReStatus(v)) => {
                    let s = String::from_utf8_lossy(&v);
                    println!("\u{1b}[1mNew status\u{1b}[0m: {s}");
                }
                Some(e) => {
                    eprintln!(
                        "\u{1b}[1;31merror\u{1b}[0;1m: server returned an error\u{1b}[0m: {e}"
                    );
                }
                None => {
                    eprintln!(
                        "\u{1b}[1;31merror\u{1b}[0;1m: server returned incorrect response\u{1b}[0m"
                    );
                }
            }
        }
    }
}
