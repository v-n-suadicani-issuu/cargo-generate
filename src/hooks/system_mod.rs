use rhai::{Dynamic, EvalAltResult, Module};
use std::process::Command;

use crate::{
    interactive::prompt_for_variable,
    project_variables::{StringEntry, TemplateSlots, VarInfo},
};

type Result<T> = std::result::Result<T, Box<EvalAltResult>>;

/// Creates the system module, containing the `command` function,
/// which allows you to run system command.
pub fn create_module(allow_commands: bool, silent: bool) -> Module {
    let mut module = Module::new();

    module.set_native_fn("command", move |name: &str, commands_args: rhai::Array| {
        run_command(name, commands_args, allow_commands, silent)
    });

    module
}

fn run_command(
    name: &str,
    args: rhai::Array,
    allow_commands: bool,
    silent: bool,
) -> Result<Dynamic> {
    let args: Vec<String> = args.into_iter().map(|arg| arg.to_string()).collect();

    // If --allow-commands is false, we need to prompt. But we shouldn't if we're in silent mode.
    if !allow_commands && silent {
        return Err("Cannot prompt for system command confirmation in silent mode. Use --allow-commands if you want to allow the template to run system commands in silent mode.".into());
    }

    // If the user specified the --allow-commands flag, don't prompt.
    let should_run = allow_commands
        || {
            let joined_args = args.join(" ");
            let prompt = format!("The template is requesting to run the following command. Do you agree?\n{name} {joined_args}");

            // Prompt the user for whether they actually want to run the command.
            let value = prompt_for_variable(&TemplateSlots {
                prompt,
                var_name: "".into(),
                var_info: VarInfo::String {
                    entry: Box::new(StringEntry {
                        default: Some("no".into()),
                        choices: Some(vec!["yes".into(), "no".into()]),
                        regex: None,
                    }),
                },
            });

            // Only accept clearly positive affirmations.
            matches!(
                value.map(|s| s.trim().to_ascii_lowercase()).as_deref(),
                Ok("yes" | "y")
            )
        };

    if !should_run {
        return Err("User denied execution of system command.".into());
    }

    let output = Command::new(name).args(args).output();

    match output {
        Ok(_) => Ok(Dynamic::UNIT),
        Err(e) => Err(format!("System command failed to execute: {e}").into()),
    }
}
