use std::{borrow::Borrow, cell::RefCell, collections::HashMap, env, fs, process::Command};

use abi_stable::std_types::{
    ROption::{self, RNone, RSome},
    RString, RVec,
};
use anyrun_plugin::*;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;

#[derive(Deserialize)]
struct Shortcut {
    command: RString,
    icon: ROption<RString>,
}

#[derive(Deserialize, Default)]
struct Config {
    shell: ROption<RString>,
    shortcuts: HashMap<RString, Shortcut>,
}

struct State {
    config: Config,
    shell: RString,
}

#[init]
fn init(config_dir: RString) -> State {
    let config = match fs::read_to_string(format!("{}/shell-shortcuts.ron", config_dir)) {
        Ok(content) => match ron::from_str::<Config>(&content) {
            Ok(config) => config,
            Err(why) => {
                println!(
                    "[Shell Shortcuts] Failed to parse '{}/shell-shortcuts.ron'",
                    config_dir
                );
                println!("[Shell Shortcuts] Error: '{}'", why);
                Config::default()
            }
        },
        Err(_) => Config::default(),
    };

    let shell = if let RSome(shell) = &config.shell {
        shell.clone()
    } else {
        env::var("SHELL")
            .unwrap_or_else(|_| "bash".to_owned())
            .into()
    };

    State { config, shell }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Shell Shortcuts".into(),
        icon: "utilities-terminal".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &State) -> RVec<Match> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().smart_case();

    state
        .config
        .shortcuts
        .iter()
        .filter(|(key, _)| matcher.fuzzy_indices(&key, &input).is_some())
        .map(|(key, val)| Match {
            title: key.clone(),
            description: RSome(val.command.clone()),
            use_pango: false,
            icon: val.icon.clone(),
            id: RNone,
        })
        .collect()
}

#[handler]
fn handler(selection: Match, state: &State) -> HandleResult {
    let command = selection.description.unwrap();
    if let Err(why) = Command::new(state.shell.as_str())
        .arg("-c")
        .arg(command.as_str())
        .spawn()
    {
        println!(
            "[Shell Shortcuts] Failed to run command: {}",
            command.as_str()
        );
        println!("[Shell Shortcuts] Error: {}", why);
    }

    HandleResult::Close
}