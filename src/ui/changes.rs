use crate::app::Command;

/// Converts staged UI edits into executable commands.
pub trait IntoCommands {
    fn into_commands(self) -> Vec<Command>;
}

/// Appends staged UI edits to an existing command list.
pub fn extend_commands<T: IntoCommands>(commands: &mut Vec<Command>, staged: T) {
    commands.extend(staged.into_commands());
}
