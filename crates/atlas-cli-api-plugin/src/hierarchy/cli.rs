use clap::{command, Command};
use convert_case::{Case, Casing};

use super::{Hierarchy, HierarchyEntry};

impl Into<clap::Command> for &Hierarchy {
    fn into(self) -> clap::Command {
        let api_root = self
            .entries
            .iter()
            .fold(Command::new("api"), |cmd, (slug, e)| {
                cmd.subcommand(e.into_command(slug))
            }).subcommand_required(true);

        let mut root = command!();
        root = root.subcommand(api_root).subcommand_required(true);
        root.build();

        root
    }
}

impl HierarchyEntry {
    fn into_command(&self, slug: &String) -> clap::Command {
        let command_name = self
            .entity_name
            .as_ref()
            .unwrap_or(slug)
            .to_case(Case::Camel);

        let mut cmd = Command::new(command_name);
        for (verb, _operation_id) in &self.verbs {
            let sub_command = Command::new(verb);


            cmd = cmd.subcommand(sub_command);
        }

        for (slug, entry) in &self.entries {
            let sub_command = entry.into_command(slug);
            cmd = cmd.subcommand(sub_command);
        }

        cmd
    }
}
