# Demonify

A tool to help manage custom daemons. This is a very crude/poor man's approach to daemon management, and should probably not be used in most environment.

## Summary

I've been wanting to move my Discord bots and some other services from Google Cloud Compute to my Raspberry Pi so I can have full control over them. Thus, I made this to allow me to add, start, remove, and kill local programs as systemd services. 

The tool works by providing a simple frontend that modifies entries in a specified `.sh` file. This file should be the target of a systemd process so that all these processes run upon bootup. As such, these processes are meant to be simple ones that don't have much dependencies in systemd.

## How to use

```
Demonify: A tool to create and manage custom daemon programs

Usage: demonify [OPTIONS] <--add|--remove|--list|--kill|--spawn> <SERVICE_FILE>

Arguments:
  <SERVICE_FILE>
          Service file to modify. The file must exist.
          
          (Note: This program will add the `#!/bin/bash` header)

Options:
  -n, --name <NAME>
          Name of the service.
          
          All spaces will be replaced with `-`

  -a, --add
          Add an entry to the service file.
          
          Name must be unique across both the service file entries AND running processes on the host machine.

  -c, --command <COMMAND>
          Shell command used to start a process. Required when adding an entry. Example: `python3 example.py`
          
          All commands will automatically have stdout/stderr info redirected to `<NAME>.log`

  -s, --spawn
          Spawns the specified process.
          
          Equivalent to calling `<COMMAND> &>> <NAME>.log` as seen in the service file.

  -r, --remove
          Remove an entry from the service file. Does not kill the process if running

  -k, --kill
          Kills the specified process.
          
          Equivalent to calling `pkill -f <NAME>` but ensures an exact name match.

  -l, --list
          Lists the name, command, and status of all entries in the service file

  -h, --help
          Print help (see a summary with '-h')
```

## Future Plans

In the future, I want to add support for creating the aformentioned systemd file so all the user would need to do is move it to `/etc/systemd/system/` and run 
- `sudo systemctl daemon-reload` 
- `sudo systemctl enable demonify.service`
- `sudo systemctl restart demonify.service`