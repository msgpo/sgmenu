use std::collections::BTreeMap;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};

use gio::prelude::*;
use gio::AppInfo;
use glib::shell_parse_argv;
use pty::fork::Fork;

pub struct Launcher{}

impl Launcher {
    pub fn new() -> Launcher {
        Launcher{}
    }
    
    fn all_applications(&self) -> Vec<AppInfo> {
        let apps = AppInfo::get_all();
        return apps
    }

    pub fn visible_applications(&self) -> Vec<AppInfo> {
        let apps = self.all_applications().into_iter().filter(|ref ai| ai.should_show()).collect::<Vec<AppInfo>>();
        return apps
    }

    pub fn desktop_applications(&self) -> BTreeMap<String, AppInfo> {
        let mut apps: BTreeMap<String, AppInfo> = BTreeMap::new();
        for appinfo in self.visible_applications() {
            if let Some(display_name) = appinfo.get_display_name() {
                apps.insert(display_name, appinfo);
            }
        }
        return apps
    }

    pub fn desktop_applications_string(&self, apps: BTreeMap<String, AppInfo>) -> String {
        let app_names = apps.keys().map(|v| v.clone()).collect::<Vec<String>>();
        return app_names.join("\n")
    }

    pub fn get_command_from_appinfo(&self, appinfo: AppInfo) -> OsString {
        match appinfo.get_commandline() {
            Some(command) => command.into_os_string(),
            None => OsString::new()
        }
    }

    pub fn split_command_args(&self, cmd: OsString) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        let cmd_string = cmd.to_str().unwrap_or("");
        if cmd_string != "" {
            let raw_args: Vec<String> = shell_parse_argv(cmd_string).unwrap();
            args = self.filter_field_codes(raw_args);
        }
        return args
    }

    fn is_field_code(&self, arg: &str) -> bool {
        let mut codes = HashSet::new();
        codes.insert("%f");
        codes.insert("%F");
        codes.insert("%u");
        codes.insert("%U");
        codes.insert("%d");
        codes.insert("%D");
        codes.insert("%n");
        codes.insert("%N");
        codes.insert("%i");
        codes.insert("%c");
        codes.insert("%k");
        codes.insert("%v");
        codes.insert("%%m");
        return codes.contains(arg);
    }

    // Strip Desktop Entry field codes from the exec command line
    fn filter_field_codes(&self, args: Vec<String>) -> Vec<String> {
        return args.into_iter().filter(|arg| !self.is_field_code(arg)).collect::<Vec<String>>();
    }

    pub fn launch(&self, cmd: OsString) {
        let args = self.split_command_args(cmd);
        info!("Executing command: {:?}", args);
        if args.len() == 0 {
            info!("Command arguments could not be parsed");
        } else if args.len() == 1 {
            Command::new(args[0].clone()).status().expect("Could not execute");
        } else if args.len() > 1 {
            let argv = args.clone().split_off(1);
            Command::new(args[0].clone())
                    .args(argv)
                    .status().expect("Could not execute");
        }
    }

    pub fn launch_pty(&self, cmd: OsString) {
        let fork = Fork::from_ptmx().unwrap();
        if let Some(mut master) = fork.is_parent().ok() {
            let mut output = String::new();
            match master.read_to_string(&mut output) {
                Ok(_nread) => info!("Child output: {}", output.trim()),
                Err(e)     => panic!("Read error error: {}", e),
            }
        } else {
            self.launch(cmd);
        }
    }

    pub fn send_application_names(&self, names: String, receiver: Vec<String>) -> Option<String> {
        let cmd = &receiver[0];
        let args = receiver.clone().split_off(1);
        let process = match Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn() {
                Err(e) => panic!("Couldn't spawn receiver command {}",
                                e.description()),
                Ok(process) => process,
            };
        match process.stdin.unwrap().write_all(names.as_bytes()) {
            Err(e) => panic!("Couldn't write to receiver command stdin: {}",
                            e.description()),
            Ok(_) => info!("Sent application names to receiver command"),
        }
        let mut choice = String::new();
        match process.stdout.unwrap().read_to_string(&mut choice) {
            Err(e) => panic!("Couldn't read receiver command stdout: {}",
                            e.description()),
            Ok(_) => {
                if !choice.is_empty() {
                    let trimmed = choice.as_str().trim().to_string();
                    return Some(trimmed)
                }
                return None
            }
        }
    }
}