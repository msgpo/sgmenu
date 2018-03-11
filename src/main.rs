#[macro_use] extern crate log;
extern crate env_logger;
extern crate gdk;
extern crate getopts;
extern crate gio;
extern crate glib;
extern crate pty;

use getopts::Options;
use std::env;
use std::ffi::OsString;

use glib::shell_parse_argv;

mod launcher;

// Temporary -- will eventually have a GUI and then offer dmenu as alternative
const DEFAULT_DMENU_COMMAND: &'static str = "rofi -dmenu -i -normal-window -theme Pop-Dark -p '> '";

// From desktopd, we only wrap to run .desktop files with this prefix in pty
const EXEC_PREFIX: &'static str = "/run/appimg/run-in-image";

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));   
}

#[allow(unused)]
fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Usage");
    opts.optflagopt("d", "dmenu", "Dmenu command", "COMMAND");
    opts.optflag("l", "list", "List desktop application");
    opts.optflagopt("c", "command", "Run a command wrapped with a pty", "COMMAND");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => {    
            error!("{}", f);
            return; 
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.opt_present("d") {
        let launcher = launcher::Launcher::new();
        let apps = launcher.desktop_applications();  
        let apps_string = launcher.desktop_applications_string(apps.clone());
        
        // Compiler erroneously reports receiver this as unused
        let mut receiver: Vec<String> = Vec::new();
        match matches.opts_str(&["d".to_string()]) {
            Some(command) => receiver = shell_parse_argv(&command).unwrap(),
            None => receiver = shell_parse_argv(DEFAULT_DMENU_COMMAND).unwrap()
        }
        match launcher.send_application_names(apps_string, receiver) {
            Some(choice) => {
                match apps.get(&choice) {
                    Some(appinfo) => {
                        let command = 
                            launcher.get_command_from_appinfo(appinfo.clone());
                        if command.to_str().unwrap().starts_with(EXEC_PREFIX) {
                            info!("Launching with pty");
                            launcher.launch_pty(command);
                        } else {
                            launcher.launch(command);
                        }
                        return
                    },
                    None => {
                        info!("No application found for choice: {}", choice.clone());
                        return
                    }
                }
            },
            None => {
                info!("No application was selected");
                return
            }
        }
    } else if matches.opt_present("c") {
        match matches.opts_str(&["c".to_string()]) {
            Some(command) => {
                let launcher = launcher::Launcher::new();
                info!("Command to run: {}", command);
                launcher.launch_pty(OsString::from(command));
                return
            },
            None => {
                print_usage(&program, opts);
                return
            }
        }
    }
    if matches.opt_present("l") {
        let launcher = launcher::Launcher::new();
        let apps = launcher.desktop_applications();
        for (name, _) in apps {
            println!("{}", name);
        }
        return;
    }
    print_usage(&program, opts);
    return;
}
