use std::process::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}

/// This function is a little janky. It creates a shell script as a side effect of execution.
pub fn execute_shell_script<S: AsRef<str>>(script: S) -> Result<ExitStatus, std::io::Error> {
    #[inline]
    fn inner(script: &str) -> Result<ExitStatus, std::io::Error> {
        #[cfg(target_os = "windows")]
        {
            use std::{io::Write, os::windows::process::CommandExt};
            let mut temp = tempfile::NamedTempFile::with_suffix(".bat")?;
            writeln!(temp, "@echo off")?;
            temp.write_all(script.as_bytes())?;
            temp.flush()?;
            let script_path = temp.path();
            let path_str = script_path.display().to_string();
            let status = Command::new(&path_str)
                .creation_flags(0x08000000) // prevent creation of terminal window.
                .status()?;
            // Prevents premature drop.
            drop(temp);
            Ok(status)
        }
        #[cfg(not(target_os = "windows"))]
        {
            unimplemented!("I don't use any OS besides Windows, so if you're encountering this issue, you'll want to write the proper code for your OS. The commented out code below is a good start.");
        }
        // I commented out this code because I think for at least Linux, you need to set the file to executable, and also might need to run with `sudo`. I don't know.
        // In other words, I don't have the environment to fiddle around to figure out the proper way to do this, so you're on your own if you run into this wall.
        // #[cfg(any(target_os = "linux", target_os = "macos"))]
        // {
        //     use std::io::Write;
    
        //     let mut temp = tempfile::NamedTempFile::with_suffix(".sh")?;
        //     temp.write_all(script.as_bytes())?;
        //     temp.flush()?;
        //     let script_path = temp.path();
        //     let path_str = script_path.display().to_string();
        //     let status = Command::new("sh")
        //         .arg(&path_str)
        //         .status()?;
        //     // Prevents premature drop.
        //     drop(temp);
        //     Ok(status)
        // }
    }
    inner(script.as_ref())
}

/// Command must be a single line, and must be valid for the system shell (`cmd` on Windows, `sh` on Linux. Probably `sh` on MacOS, too,
/// but I don't use MacOS, so you'll have to implement that yourself)
pub fn shell_command<S: AsRef<str>>(command: S) -> Option<Command> {
    fn inner(command: &str) -> Option<Command> {
        let args = shlex::split(command)?;
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.arg("/C");
            cmd.args(&args);
            cmd.creation_flags(1<<27);
            Some(cmd)
        }
        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new("sh");
            shell_command.arg("-c");
            shell_command.args(&args);
            Some(shell_command)
        }
        #[cfg(not(target_os = "windows"))]
        {
            unimplemented!("I didn't implement shell_command() for this target.");
        }
    }
    inner(command.as_ref())
}

pub fn exec_shell<S: AsRef<str>>(command: S) -> Result<ExitStatus, ExecError> {
    fn inner(command: &str) -> Result<ExitStatus, ExecError> {
        let mut cmd = shell_command(command).ok_or_else(|| ExecError::InvalidCommand(command.to_owned()))?;
        Ok(cmd.status()?)
    }
    inner(command.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn execute_test() {
        // execute("code run.bat").expect("Failed to execute.");
        // system(r#"code "C:\Users\derek\Documents\code\rust\bourne""#);
        exec_shell(r#"explorer.exe "." && echo test"#).expect("Failed to execute.");
    }
}