use clap::Command;

use crate::core::types::HelpTopic;
pub fn help_command(topic: HelpTopic) -> Command {
    crate::app::command_registry::help_command_for_topic(topic)
}
