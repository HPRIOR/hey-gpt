use std::{error::Error, fmt::Display, ops::Sub};

use async_trait::async_trait;
use chrono::Duration;
use log::debug;

use crate::{
    effect::{ChatRequestInput, EditRequestInput, Effects, MemQueryOpt, QueryWindow},
    model::{ChatData, EditData, Mode, Model},
};

use super::{success::SuccessState, Action};

pub struct RequestState(pub Effects);

#[async_trait]
impl Action for RequestState {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>> {
        let mode = &model.mode;
        match mode {
            Mode::Chat(_) => Ok((Box::new(ChatState { effects: self.0 }), model)),
            Mode::Edit(_) => Ok((Box::new(EditState { effects: self.0 }), model)),
        }
    }

    fn _type(&self) -> String {
        String::from("Request")
    }
}

struct PotentialPrompt<'a> {
    prompt: &'a String,
    data_prompt_result: &'a Option<String>,
    std_in: Option<&'a String>,
}

fn get_potential_prompts(model: &Model) -> PotentialPrompt {
    let prompt = &model.prompt.prompt;
    let data_prompt_result = &model.prompt.generated_data;
    let std_in = match &model.mode {
        Mode::Chat(ChatData::DataStdIn(data)) => Some(data),
        Mode::Edit(EditData::DataStdIn(data)) => Some(data),
        _ => None,
    };
    PotentialPrompt {
        prompt,
        data_prompt_result,
        std_in,
    }
}

pub struct EditState {
    effects: Effects,
}

#[derive(Debug)]
struct EditStateError(String);

impl Display for EditStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for EditStateError {}

#[async_trait]
impl Action for EditState {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>> {
        match get_potential_prompts(&model) {
            PotentialPrompt {
                data_prompt_result: None,
                std_in: None,
                ..
            } => Err(Box::new(EditStateError(String::from(
                "No input data given to edit request",
            )))),
            PotentialPrompt {
                prompt,
                data_prompt_result: Some(input),
                ..
            }
            | PotentialPrompt {
                prompt,
                std_in: Some(input),
                ..
            } => {
                let request_input = EditRequestInput {
                    instruction: prompt.clone(),
                    input: input.clone(),
                };

                let response = self
                    .effects
                    .requester
                    .edit_request_stream(request_input, &model.algo)
                    .await?;

                let data = self.effects.displayer.print_stream(response).await;
                Ok((
                    Box::new(SuccessState(self.effects)),
                    model.with_edit_response(data),
                ))
            }
        }
    }

    fn _type(&self) -> String {
        String::from("Edit")
    }
}

pub struct ChatState {
    effects: Effects,
}

#[async_trait]
impl Action for ChatState {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>> {
        let chat_request = match get_potential_prompts(&model) {
            PotentialPrompt {
                prompt,
                data_prompt_result: Some(data),
                ..
            }
            | PotentialPrompt {
                prompt,
                std_in: Some(data),
                ..
            } if !data.is_empty() => format!("{}. Use this input: {}.", prompt, data),
            PotentialPrompt { prompt, .. } => prompt.to_string(),
        };

        debug!("Retrieving chat history: ");
        let convo_history = if model.memory.convo_len > 0 {
            // history file should be here as check done in argument parsing
            self.effects
                .history
                .get_history( model.memory.convo_len)
                .await?
        } else {
            Default::default()
        };

        debug!("Retrieved convo history: {:#?}", convo_history);

        let memories = {
            if model.memory.enabled {
                let convo_query_window = QueryWindow{
                    min: None,
                    // this is a bit messy, ideally should be from the first message before the
                    // window
                    max: convo_history.first().map(|dialogue| dialogue.created_at.sub(Duration::seconds(1)))
                };

                let convo_query_input = MemQueryOpt {
                    category: model.memory.convo.to_string(),
                    query_window: convo_query_window,
                };

                let mut mem_query_input: Vec<MemQueryOpt> = model
                    .memory
                    .memories
                    .iter()
                    .map(|memory| MemQueryOpt {
                        category: memory.to_string(),
                        query_window: QueryWindow {
                            max: None,
                            min: None,
                        },
                    })
                    .collect();

                mem_query_input.push(convo_query_input);

                debug!("Retrieving memories from db with the following input: query - {:#?}; query options - {:#?}", chat_request, mem_query_input);
                self.effects
                    .context
                    .query(&chat_request, &mem_query_input)
                    .await?
            } else {
                Default::default()
            }
        };

        debug!("Found {} memories. {:#?}", memories.len(), memories);

        debug!("Constructing system message");
        let mut system_msg = {
            let memory_string = memories
                .iter()
                .map(|mem| {
                    format!(
                        "[author: {}, category: {}, created_at: {}] {}\n",
                        mem.author, mem.category, mem.created_at, mem.text
                    )
                })
                .fold(String::from(""), |mut acc, i| {
                    acc.push_str(i.as_str());
                    acc
                });

            let memory_msg = if !memory_string.is_empty() {
                format!("Below is a list of text related to the current query, it has metadata prepended between the square braces: {}", memory_string)
            } else {
                "".to_string()
            };

            let msg = format!("{}. {}", model.prompt.act_as, memory_msg);
            vec![ChatRequestInput {
                role: "system".to_string(),
                content: msg,
            }]
        };

        debug!("Constructing conversation message");
        let mut conversation: Vec<ChatRequestInput> = convo_history
            .iter()
            .map(|x| ChatRequestInput {
                role: x.author.to_string(),
                content: x.content.to_string(),
            })
            .collect();

        debug!("Constructing current query");
        let mut current = vec![ChatRequestInput {
            role: "user".to_string(),
            content: chat_request.to_string(),
        }];

        let request = {
            let mut result = vec![];
            result.append(&mut system_msg);
            result.append(&mut conversation);
            result.append(&mut current);
            result
        };

        debug!("Sending query");
        let response_stream = self
            .effects
            .requester
            .chat_request_stream(&request, &model)
            .await?;

        let result = self.effects.displayer.print_stream(response_stream).await;
        Ok((
            Box::new(SuccessState(self.effects)),
            model
                .with_chat_response(result)
                .with_chat_prompt(chat_request.to_string()),
        ))
    }

    fn _type(&self) -> String {
        String::from("Chat")
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn hello_world() {

    }
}
