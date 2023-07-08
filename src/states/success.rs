use std::{error::Error, process::exit};

use async_trait::async_trait;
use log::debug;

use crate::{
    effect::{Effects, LongMemSaveInp, ShortMemInput},
    model::{Mode, Model},
};

use super::Action;

pub struct SuccessState(pub Effects);

#[async_trait]
impl Action for SuccessState {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>> {
        // save memory if in chat mode
        match &model.mode {
            Mode::Chat(_) => {
                let has_chat_output = model
                    .output
                    .chat_results
                    .as_ref()
                    .map(|r| !r.is_empty())
                    .unwrap_or(false);

                if has_chat_output {
                    // save chat history to long term storage
                    let prompt = model.prompt.final_chat_prompt.unwrap_or("".to_string());
                    let response = model
                        .output
                        .chat_results
                        .map(|results| results[0].clone()) // multiple responses not yet implementated
                        .unwrap_or("".to_string());

                    if model.memory.enabled {
                        debug!("Saving prompt and response to database");
                        let user_input = LongMemSaveInp {
                            text: prompt.clone(),
                            author: "user".to_string(),
                        };

                        let assistant_response = LongMemSaveInp {
                            text: response.clone(),
                            author: "assistant".to_string(),
                        };

                        self.0
                            .context
                            .save(&[user_input, assistant_response], &model.memory.convo)
                            .await?;
                    }

                    // save chat history to short term storage
                    let user_input = ShortMemInput {
                        author: "user".to_string(),
                        content: prompt,
                    };

                    let assistant_response = ShortMemInput {
                        author: "assistant".to_string(),
                        content: response,
                    };
                    self.0
                        .history
                        .save_history(&[user_input, assistant_response])
                        .await?;
                };
            }
            Mode::Edit(_) => (),
        }
        exit(0);
    }

    fn _type(&self) -> String {
        String::from("Success")
    }
}
