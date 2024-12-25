use colored::Colorize;
use regex::Regex;
use std::{
    collections::HashMap,
    process::{exit, Command},
};

use clap::{arg, command, ArgGroup, Parser};

/// Demonify: A tool to create and manage custom daemon programs.
#[derive(Debug, Parser)]
#[command(about)]
#[clap(group(
    ArgGroup::new("action")
        .required(true)
        .args(&["add_service", "remove_service"]),
))]
struct Args {
    /// Service file to modify. The file must exist.
    ///
    /// (Note: This program will add the `#!/bin/bash` header)
    service_file: String,

    /// Name of the service, which must be unique across other entries AND host computer's running processes
    ///
    /// (Note: all spaces will be replaced with `-`)
    name: String,

    /// Add a service to the service file.
    #[arg(short, long = "add", requires = "command")]
    add_service: bool,

    /// Associated shell command to be run.
    ///
    /// (Note: all commands will automatically have stdout/stderr info redirected to `<NAME>.log`)
    #[arg(short, long)]
    command: Option<String>,

    /// Additionally spawns the process after adding its entry to the service file.
    ///
    /// Equivalent to calling `<COMMAND> 2>&1 >> <NAME>.log`.
    #[arg(short, long)]
    spawn: bool,

    /// Remove an entry from the service file.
    #[arg(short, long = "remove")]
    remove_service: bool,

    /// Additionally kills the process after removing its entry from the service file.
    ///
    /// Equivalent to calling `pkill -f <NAME>`.
    #[arg(short, long, requires = "remove_service")]
    kill: bool,
}

fn parse_service_file(path: &str) -> HashMap<String, String> {
    let contents = std::fs::read(path)
        .unwrap_or_else(|_| {
            println!("Error reading from file. Does the file exist?");
            exit(1);
        })
        .iter()
        .map(|b| char::from(*b))
        .collect::<String>();
    let contents: Vec<&str> = contents.split('\n').collect();

    let re = Regex::new(
        r##"^bash -c "exec -a \b([\w\-]+)\b (.+) 2>&1 > \b([\w\-]+)\b\.log &" # ([\w\-]+)"##,
    )
    .expect("Regex compilation error.");

    let mut map = HashMap::new();

    contents[1..].iter().for_each(|&s| {
        let captures = re.captures(s).unwrap_or_else(|| {
            println!("File is poorly formatted. Aborting.");
            exit(1);
        });
        if captures[1] != captures[3] || captures[1] != captures[4] {
            println!("File is poorly formatted. Aborting");
            exit(1);
        }
        map.insert(
            captures.get(1).unwrap().as_str().to_string(),
            captures.get(2).unwrap().as_str().to_string(),
        );
    });
    map
}

fn main() {
    let mut args = Args::parse();
    if args.kill && !args.remove_service {
        println!(
            "The {} flag can only be called when adding a service with {}",
            "--kill".green(),
            "--remove".green()
        );
        exit(1);
    }

    if args.spawn && !args.add_service {
        println!(
            "The {} flag can only be called when adding a service with {}",
            "--spawn".green(),
            "--add".green()
        );
        exit(1);
    }

    args.name = args.name.replace(" ", "-");

    let mut map = parse_service_file(&args.service_file);

    if args.add_service {
        // Add Service
        if map.contains_key(&args.name) {
            println!(
                "An existing entry for the service name `{}` was detected. It's current command is `{}`.",
                &args.name,
                map.get(&args.name).unwrap()
            );
            println!("To update the value, use --remove {} first.", &args.name);
        } else {
            let system_info = sysinfo::System::new_all();
            if system_info
                .processes()
                .iter()
                .find(|&(_pid, process)| {
                    process
                        .cmd()
                        .get(0)
                        .is_some_and(|s| s.to_string_lossy() == *args.name)
                })
                .is_some()
            {
                println!("There is already an existing process with the name `{}`.\nTo avoid conflicts, please choose a different name.", &args.name);
                exit(1);
            }

            map.insert(args.name.clone(), args.command.clone().unwrap());
            println!("Service entry `{}` successfully added.", &args.name);

            if args.spawn {
                let bash_command = format!(
                    r#"exec -a {} {} 2>&1 >> {}.log &"#,
                    &args.name,
                    &args.command.clone().unwrap(),
                    &args.name
                );

                match Command::new("bash").arg("-c").arg(&bash_command).status() {
                    Ok(_) => println!("Successfully spawned process `{}`", &args.name),
                    Err(e) => println!(
                        "Failed to spawn process `{}` with command `{}`. Error: {}",
                        &args.name, bash_command, e
                    ),
                }
            }
        }
    } else {
        // Remove Service
        if !map.contains_key(&args.name) {
            println!("No entry was found for the service name `{}`", &args.name);
        } else {
            map.remove(&args.name);
            println!("Service entry `{}` successfully removed.", &args.name);

            if args.kill {
                match sysinfo::System::new_all().processes().iter().find(|&(_pid, process)| {
                    process
                        .cmd()
                        .get(0)
                        .is_some_and(|s| s.to_string_lossy() == *args.name)
                }) {
                    Some((_pid, process)) => {
                        if process.kill() {
                            println!("Successfully killed process `{}`", &args.name);
                        } else {
                            println!("Unable to kill process `{}`", &args.name);
                        }
                    }
                    None => println!("Unable to locate process `{}`. Was it started?", &args.name),
                };
            }
        }
    }

    let mut to_write = vec!["#!/bin/bash".to_string()];
    map.iter().for_each(|(name, command)| {
        to_write.push(format!(
            r##"bash -c "exec -a {} {} 2>&1 > {}.log &" # {}"##,
            name, command, name, name
        ));
    });

    let buffer = to_write.join("\n");
    std::fs::write(args.service_file, &buffer[..]).unwrap_or_else(|_| {
        println!("Error writing to file. Does the file exist?");
        exit(1);
    });
}
