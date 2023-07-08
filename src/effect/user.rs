use core::time;
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    process::Command,
    thread,
};

use tempfile::NamedTempFile;
use termion::{event::Key, input::TermRead};

use super::{DisplayEffect, InteractionEffect, UserCycleResponse};

pub struct User(pub Box<dyn DisplayEffect>);

impl InteractionEffect for User {
    fn elicit_cycle_response(&self, user_prompt: &str) -> UserCycleResponse {
        self.0.print(user_prompt);

        let mut stdin = termion::async_stdin().keys();

        loop {
            let input = stdin.next();

            if let Some(Ok(key)) = input {
                match key {
                    Key::Char('\n') | Key::Char('\r') => break UserCycleResponse::Accept,
                    Key::Char('e') => break UserCycleResponse::Edit,
                    Key::Right | Key::Char('l') => break UserCycleResponse::NextRight,
                    Key::Left | Key::Char('h') => break UserCycleResponse::NextLeft,
                    _ => (),
                }
            }
            thread::sleep(time::Duration::from_millis(50));
        }
    }

    fn edit_data_gen_prompt(&self, initial_prompt: &str) -> Result<String, Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(initial_prompt.as_bytes())?;
        temp_file.flush()?;

        let temp_file_path = temp_file.path().to_str().unwrap();

        let status = Command::new("nvim").arg(temp_file_path).status()?;

        if !status.success() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "NeoVim did not exit successfully",
            )));
        }

        let mut edited_content = String::new();
        let mut temp_file_read = File::open(temp_file_path)?;
        temp_file_read.read_to_string(&mut edited_content)?;

        temp_file.close()?;

        if edited_content == initial_prompt {
            Ok(initial_prompt.to_string())
        } else {
            Ok(edited_content)
        }
    }
}
