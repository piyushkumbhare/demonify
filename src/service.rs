use std::{collections::HashMap, process::Command};

use colored::Colorize;
use regex::Regex;

use crate::*;

/// Parses the service file at path `args.service_file` and attempts to create a
/// map representing the configuration, using Name as a key and Command as a value.
#[inline]
pub fn parse_service_file(args: &Args) -> HashMap<String, String> {
    let contents = std::fs::read(&args.service_file)
        .unwrap_or_else(|_| {
            println!("Error reading from file. Does the file exist?");
            exit(1);
        })
        .iter()
        .map(|b| char::from(*b))
        .collect::<String>();

    let contents: Vec<&str> = contents.split('\n').filter(|line| line.len() > 0).collect();

    let re = Regex::new(
        r##"bash -c "exec -a \b([\w\-]+)\b (.+) &>> \b([\w\-]+)\b\.log &" # ([\w\-]+)"##,
    )
    .expect("Regex compilation error.");

    let mut map = HashMap::new();

    contents.get(1..).unwrap_or(&[]).iter().for_each(|&s| {
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

/// Adds the specified service to `map`, using `args.name` as a key and `args.command` as a value.
/// Optionally attempts to spawn the process if `args.spawn == true`.
#[inline]
pub fn add_service(args: &Args, map: &mut HashMap<String, String>) {
    let name = args.name.clone().unwrap();
    if map.contains_key(&name) {
        println!(
            "An existing entry for the service name {} was detected. It's current command is {}.",
            &name,
            map.get(&name).unwrap()
        );
        println!("To update the value, use --remove {} first.", &name.bold());
    } else if check_service_active(&name) {
        println!("There is already an existing process with the name {}.\nTo avoid conflicts, please choose a different name.", &name.bold());
        exit(1);
    } else {
        map.insert(name.clone(), args.command.clone().unwrap());
        println!("Service entry {} successfully added.", &name.bold());
    }
}

/// Removes the specified service from `map`, using `args.name` as a key.
/// Optionally attempts to kill the process if `args.kill == true`.
#[inline]
pub fn remove_service(args: &Args, map: &mut HashMap<String, String>) {
    let name = args.name.clone().unwrap();
    if !map.contains_key(&name) {
        println!("No entry was found for the service name {}", &name.bold());
    } else {
        map.remove(&name);
        println!("Service entry {} successfully removed.", &name.bold());
    }
}

/// Attempts to spawn the service with name `args.name`.
#[inline]
pub fn spawn_service(args: &Args, map: &mut HashMap<String, String>) {
    let name = args.name.clone().unwrap();
    if check_service_active(&name) {
        println!("Service {} is already active.", &name.bold());
        exit(1);
    }
    let bash_command = format!(
        "exec -a {} {} &>> {}.log &",
        &name,
        &map.get(&name).unwrap_or_else(|| {
            println!("No entry was found for the service name {}", &name.bold());
            exit(1);
        }),
        &name
    );

    match Command::new("bash").arg("-c").arg(&bash_command).status() {
        Ok(_) => println!("Successfully spawned process {}", &name.bold()),
        Err(e) => println!(
            "Failed to spawn process {} with command {}. Error: {}",
            &name, bash_command, e
        ),
    }
}

/// Attempts to kill the service with name `args.name`
#[inline]
pub fn kill_service(args: &Args) {
    let name = args.name.clone().unwrap();
    if !check_service_active(&name) {
        println!("Could not find service {}. Was it started?", &name.bold());
    } else {
        match sysinfo::System::new_all()
            .processes()
            .iter()
            .find(|(_pid, process)| {
                process
                    .cmd()
                    .first()
                    .is_some_and(|s| s.to_string_lossy() == *name)
            })
            .is_some_and(|(_pid, process)| process.kill())
        {
            true => println!("Successfully killed service {}", &name.bold()),
            false => println!(
                "Failed to kill service {}. Perhaps try manually killing it with {} ?",
                &name.bold(),
                format!("pkill -f {}", name).bold().italic(),
            ),
        }
    }
}

/// Prints the `name, command, status` of each entry.
#[inline]
pub fn list_service(map: &mut HashMap<String, String>) {
    println!("{: <16}\t{: <40} {: >8}", "Name", "Command", "Status");
    println!();
    let mut total_active_processes: usize = 0;
    map.iter().for_each(|(name, command)| {
        let active = match check_service_active(name) {
            true => {
                total_active_processes += 1;
                "Active".green()
            }
            false => "Inactive".red(),
        };

        println!(
            "{: <16}\t{: <40} {: >8}",
            name.bold(),
            command.italic().bold(),
            active.bold()
        );
    });
    println!();
    println!(
        "({} entries | {} active)",
        map.len(),
        total_active_processes
    );
}

/// Returns true if a process with name `name` is found.
fn check_service_active(name: &str) -> bool {
    let bash_command = format!("ps -eo args | awk '{{ print $1 }}' | grep -q ^{}$", &name);
    Command::new("bash")
        .arg("-c")
        .arg(bash_command)
        .status()
        .is_ok_and(|c| c.success())
}

/// Writes the new config of `map` to the service file at path `args.service_file`
#[inline]
pub fn write_service_file(args: &Args, map: &mut HashMap<String, String>) {
    let mut to_write = vec!["#!/bin/bash".to_string()];
    map.iter().for_each(|(name, command)| {
        to_write.push(format!(
            r##"bash -c "exec -a {} {} &>> {}.log &" # {}"##,
            name, command, name, name
        ));
    });

    let buffer = to_write.join("\n");
    std::fs::write(&args.service_file, &buffer[..]).unwrap_or_else(|_| {
        println!("Error writing to file. Does the file exist?");
        exit(1);
    });
}
