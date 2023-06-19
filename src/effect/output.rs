use std::{
    io::{self, stderr, stdout, Write},
    pin::Pin,
};

use async_trait::async_trait;
use futures::{Stream, StreamExt};

use super::DisplayEffect;

pub struct Output;

#[async_trait]
impl DisplayEffect for Output {
    /// Consumes Vec<String> stream printing the first item in the Vector. Stores output in a
    /// vector by concatonating the strings
    async fn print_stream(
        &self,
        mut input: Pin<Box<dyn Stream<Item = Vec<String>> + Send>>,
    ) -> Vec<String> {
        let mut responses: Vec<String> = vec![];
        while let Some(item) = input.next().await {
            item.iter().enumerate().for_each(|(i, s)| {
                if i == 0 {
                    print!("{}", s);
                    io::stdout().flush().unwrap();
                };
                responses
                    .get_mut(i)
                    .map(|accumulated| {
                        accumulated.push_str(s);
                    })
                    .unwrap_or_else(|| responses.insert(i, s.to_string()));
            })
        }
        // newline at end of output
        println!();
        io::stdout().flush().unwrap();
        responses
    }

    fn print(&self, input: &str) {
        println!("{}", input);
        stdout().flush().unwrap();
    }

    fn eprint(&self, input: &str) {
        eprintln!("{}", input);
        stderr().flush().unwrap();
    }
}
