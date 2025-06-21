use std::{ffi::{CStr, CString}, process::*};

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
}

// This isn't actually as useful as I was hoping since it opens a terminal, which isn't what is wanted.
pub fn system<S: AsRef<str>>(command: S) -> i32 {
    fn inner(command: &str) -> i32 {
        let c_str = CString::new(command).expect("Failed to create CString.");
        unsafe {
            libc::system(c_str.as_ptr())
        }
    }
    inner(command.as_ref())
}

pub fn non_blocking_system<S: AsRef<str>>(command: S) {
    fn inner(command: &str) {
        let c_str = CString::new(command).expect("Failed to create CString.");
        std::thread::spawn(move || {
            unsafe {
                libc::system(c_str.as_ptr());
            }
            drop(c_str);
        });
    }
    inner(command.as_ref());
}

pub fn execute<S: AsRef<str>>(script: S) -> Result<ExitStatus, std::io::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn execute_test() {
        // execute("code run.bat").expect("Failed to execute.");
        system(r#"code "C:\Users\derek\Documents\code\rust\bourne""#);
    }
}