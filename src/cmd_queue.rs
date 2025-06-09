
pub struct Commands<T> {
    commands: Vec<T>,
}

pub struct CommandBufRef<'a, T> {
    commands: Option<&'a mut Vec<T>>,
}

impl<'a, T> CommandBufRef<'a, T> {
    #[inline]
    pub fn push(&mut self, command: T) {
        if let Some(ref mut commands) = self.commands {
            commands.push(command);
        }
    }
}

impl<T> Commands<T> {
    pub const fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn dummy_sender() -> CommandBufRef<'static, T> {
        CommandBufRef { commands: None }
    }

    pub fn sender(&mut self) -> CommandBufRef<'_, T> {
        CommandBufRef {
            commands: Some(&mut self.commands),
        }
    }

    pub fn finish(self) -> Vec<T> {
        self.commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_buffer_test() {
        let mut commands = Commands::new();
        fn submit(mut commands: CommandBufRef<'_, String>) {
            commands.push(format!("Hello, world!"));
            commands.push(format!("This is a test."));
        }
        submit(commands.sender());
        for command in commands.finish() {
            println!("{command}");
        }
    }
}