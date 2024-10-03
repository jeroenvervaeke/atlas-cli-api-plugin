use std::collections::BTreeMap;

use anyhow::{bail, Result};
use clap::{command, Arg, Command};
use convert_case::{Case, Casing};
use openapiv3::OpenAPI;
use operation::Operation;
use serde_yaml;

mod operation;
mod path;

fn main() -> Result<()> {
    let data = include_str!("../../../raw-spec.yaml");
    let spec: OpenAPI = serde_yaml::from_str(data).expect("Could not deserialize input");

    let mut operations = Vec::new();
    for operation_result in spec
        .operations()
        .map(|(path, verb, operation)| Operation::new(&spec, path, verb, operation))
    {
        match operation_result {
            Ok(operation) => operations.push(operation),
            Err(e) => match e {
                operation::OperationCreationError::UnexpectedNumberOfTags { got, path, verb } => {
                    eprintln!(
                        "Skipping endpoint: {verb} {path}, number of tags expected: 1, got: {got}"
                    );
                }
                e => bail!("failed to parse OAS spec: {e}"),
            },
        }
    }

    // Group operations per tag
    let mut grouped_operations = BTreeMap::<String, Vec<Operation>>::new();
    for operation in operations {
        if let Some(group) = grouped_operations.get_mut(&operation.tag) {
            group.push(operation);
        } else {
            grouped_operations.insert(operation.tag.clone(), vec![operation]);
        }
    }

    let mut api_root = Command::new("api").subcommand_required(true);
    for (group_name, operations) in &grouped_operations {
        let mut group_cmd = Command::new(
            group_name
                .to_case(Case::Kebab)
                .trim_start_matches("atlas-")
                .to_owned(),
        );

        if let Some(tag_description) = spec
            .tags
            .iter()
            .find_map(|t| (&t.name == group_name).then(|| t.description.as_ref().to_owned()))
            .flatten()
        {
            if let Some((first_sentence, _rest)) = tag_description.split_once('.') {
                group_cmd = group_cmd.about(first_sentence.trim().to_lowercase());
            }

            group_cmd = group_cmd.long_about(tag_description);
        }

        for operation in operations {
            let mut command = Command::new(operation.operation_id.to_owned());
            if let Some(description) = operation.description.as_ref() {
                if let Some((first_sentence, _rest)) = description.split_once('.') {
                    command = command.about(first_sentence.trim().to_lowercase());
                }

                command = command.long_about(description.to_owned());
            }

            for flag in &operation.flags {
                let mut arg = Arg::new(flag.name.to_owned()).long(flag.name.to_owned());
                if let Some(description) = &flag.description {
                    arg = arg.help(description.to_owned());
                }
                command = command.arg(arg);
            }

            group_cmd = group_cmd.subcommand(command);
        }

        api_root = api_root.subcommand(group_cmd);
    }

    let mut cli = command!().subcommand(api_root).subcommand_required(true);
    cli.build();

    //print_command_tree(&cli, "", true);
    let matches = cli.get_matches();

    Ok(())
}
/*
fn print_command_tree(cmd: &Command, prefix: &str, last: bool) {
    let branch = if last { "└── " } else { "├── " };
    println!("{}{}{}", prefix, branch, cmd.get_name());

    let child_prefix = format!("{}{}   ", prefix, if last { " " } else { "│" });

    let subcommands: Vec<_> = cmd
        .get_subcommands()
        .filter(|c| c.get_name() != "help")
        .collect();

    for (index, subcmd) in subcommands.iter().enumerate() {
        let is_last = index == subcommands.len() - 1;
        print_command_tree(subcmd, &child_prefix, is_last);
    }
}
 */
