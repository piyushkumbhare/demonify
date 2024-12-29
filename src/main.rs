#![cfg(target_os = "linux")]

use std::process::exit;
mod service;
use service::*;

use clap::{arg, command, ArgGroup, Parser};

/// Demonify: A tool to create and manage custom daemon programs.
#[derive(Debug, Parser)]
#[command(about)]
#[clap(group(
    ArgGroup::new("action")
        .required(true)
        .args(&["add", "remove", "list", "kill", "spawn"])
))]
struct Args {
    /// Service file to modify. The file must exist.
    ///
    /// (Note: This program will add the `#!/bin/bash` header)
    service_file: String,

    /// Name of the service.
    /// 
    /// All spaces will be replaced with `-`
    #[arg(short, long)]
    name: Option<String>,

    /// Add an entry to the service file. 
    /// 
    /// Name must be unique across both the service file entries AND running processes on the host machine.
    #[arg(short, long, requires = "command", requires = "name")]
    add: bool,

    /// Shell command used to start a process. Required when adding an entry. Example: `python3 example.py`
    ///
    /// All commands will automatically have stdout/stderr info redirected to `<NAME>.log`
    #[arg(short, long)]
    command: Option<String>,

    /// Spawns the specified process.
    ///
    /// Equivalent to calling `<COMMAND> &>> <NAME>.log` as seen in the service file.
    #[arg(short, long, requires = "name")]
    spawn: bool,

    /// Remove an entry from the service file. Does not kill the process if running.
    #[arg(short, long)]
    remove: bool,

    /// Kills the specified process.
    ///
    /// Equivalent to calling `pkill -f <NAME>` but ensures an exact name match.
    #[arg(short, long, requires = "name")]
    kill: bool,

    /// Lists the name, command, and status of all entries in the service file.
    #[arg(short, long)]
    list: bool,
}

fn main() {
    let mut args = Args::parse();

    // Replace all " " with "-" for bash consistency
    args.name = args.name.map(|n| n.replace(" ", "-").to_lowercase());

    if args.name.clone().is_some_and(|s| s.len() > 15) {
        println!("Process names have a maximum length of 15. Please use a shorter process name.");
        exit(1);
    }
    // Parse service file for current processes. HashMap<Name, Command>
    let mut map = parse_service_file(&args);

    if args.add {
        add_service(&args, &mut map);
        write_service_file(&args, &mut map);
    } else if args.remove {
        remove_service(&args, &mut map);
        write_service_file(&args, &mut map);
    } else if args.list {
        list_service(&mut map);
    }

    if args.spawn {
        spawn_service(&args, &mut map);
    } else if args.kill {
        kill_service(&args);
    }
}
