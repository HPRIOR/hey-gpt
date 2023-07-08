use std::fmt::Display;
use std::{error::Error, pin::Pin};

use crate::data::dtos::{
    ChatRequestDTO, ChatRequestMsgDTO, EditRequestDTO, EditResponseDTO, StreamChatResponseDTO,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt, stream};
use log::debug;
use reqwest::Client;

use crate::data::model::Model;
use crate::{utils::Transpose, COMPLETION_URL, EDIT_URL};

use super::{AiRequestEffect, ChatRequestInput, EditRequestInput};

pub struct GptRequest {
    client: Client,
    auth_token: String,
    model: Model,
}

impl GptRequest {
    pub fn new(client: Client, auth_token: String, model: Model) -> Self {
        Self {
            client,
            auth_token,
            model,
        }
    }
}

#[derive(Debug)]
pub struct StreamChatResponseDeserialiseError(pub Vec<String>);
impl Error for StreamChatResponseDeserialiseError {}
impl Display for StreamChatResponseDeserialiseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error occured trying to deserialise StreamChatResponseDTO: {:#?} ",
            self.0
        )
    }
}

fn parse_byte_to_response_dto(str_from_bytes: &str) -> Vec<StreamChatResponseDTO> {
    let result: Vec<StreamChatResponseDTO> = str_from_bytes
        .split("\n\n")
        .filter(|s| !s.is_empty() && str_from_bytes != "{}" && !str_from_bytes.contains("[DONE]"))
        .filter_map(|str_object| {
            debug!(
                "Attempting to convert object from byte stream: {}",
                str_object
            );
            let str_without_prefix = str_object.trim_start_matches("data: ");
            match serde_json::from_str(str_without_prefix) {
                Ok(converted) => {
                    debug!("Success!");
                    converted
                }
                Err(e) => {
                    debug!("Could not convert: {} due to {}", str_from_bytes, e);
                    None
                }
            }
        })
        .collect();

    result
}

#[async_trait]
impl AiRequestEffect for GptRequest {
    async fn chat_request_stream(
        &self,
        request: &[ChatRequestInput],
    ) -> Result<Pin<Box<dyn Stream<Item = Vec<String>> + Send + 'static>>, Box<dyn Error>> {
        let request = ChatRequestDTO {
            messages: request
                .iter()
                .map(|ChatRequestInput { role, content }| ChatRequestMsgDTO {
                    content: content.to_string(),
                    role: role.to_string(),
                })
                .collect(),
            model: self.model.algo.chat_model.to_string(),
            n: 1,
            temperature: self.model.algo.temp,
            max_tokens: self.model.algo.max_tokens,
            stream: true,
        };

        debug!("Sending chat request with request: {:#?}", request);

        let response = self
            .client
            .post(COMPLETION_URL)
            .bearer_auth(&self.auth_token)
            .json(&request)
            .send()
            .await?;

        // todo!
        let result = response
            .error_for_status()?
            .bytes_stream()
            .flat_map(|byte_result| {
                let str_from_byte = String::from_utf8(
                    byte_result
                        .expect("Did not recieve byte from byte string")
                        .to_vec(),
                )
                .expect("Could not convert byte to string when serialising StreamChatResponse");
                let response_vec =  parse_byte_to_response_dto(&str_from_byte);
                stream::iter(response_vec.into_iter())
            })
            .map(|dto: StreamChatResponseDTO| -> Vec<String> {
                dto.choices
                    .into_iter()
                    .flat_map(|choice| choice.delta.content)
                    .collect()
            });

        Ok(Box::pin(result))
    }

    async fn edit_request_stream(
        &self,
        request: EditRequestInput,
    ) -> Result<Pin<Box<dyn Stream<Item = Vec<String>> + Send + 'static>>, Box<dyn Error>> {
        // no such thing as streaming edit streams atm so just duplicating for now
        debug!(
            "Sending edit request with prompt: {:#?}\n And with data {:#?}",
            &request.instruction, &request.input
        );

        let request = {
            EditRequestDTO {
                input: request.input,
                instruction: request.instruction,
                model: self.model.algo.edit_model.clone(),
                n: 1,
                temperature: self.model.algo.temp,
            }
        };

        let response = self
            .client
            .post(EDIT_URL)
            .bearer_auth(&self.auth_token)
            .json(&request)
            .send()
            .await?;

        let result: Vec<Vec<String>> = response
            .error_for_status()?
            .json::<EditResponseDTO>()
            .await?
            .choices
            .iter()
            .map(|choice| choice.text.clone())
            .map(|content| {
                content
                    .split_whitespace()
                    .map(|x| x.to_string() + " ")
                    .collect()
            })
            .collect();

        Ok(Box::pin(futures::stream::iter(result.transpose())))
    }
}

#[cfg(test)]
mod tests {
    // use super::parse_byte_to_response_dto;
    //
    // #[test]
    // fn will_parse_empty_object() {
    //     let input = "{}";
    //     let result = parse_byte_to_response_dto(input);
    //     assert!(result.is_err())
    // }
    // #[test]
    // fn will_parse_expected_object() {
    //     let input = "data: {\"id\":\"chatcmpl-7YK1bd5RqjEmR7W5TQn3hAqoyA0Zy\",\"object\":\"chat.completion.chunk\",\"created\":1688414483,\"model\":\"gpt-3.5-turbo-0613\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\".\"},\"finish_reason\":null}]}\n\n";
    //     let result = parse_byte_to_response_dto(input);
    //     assert!(result.is_ok())
    // }
    // #[test]
    // fn will_parse_double_object() {
    //     let input =  "data: {\"id\":\"chatcmpl-7YK1bd5RqjEmR7W5TQn3hAqoyA0Zy\",\"object\":\"chat.completion.chunk\",\"created\":1688414483,\"model\":\"gpt-3.5-turbo-0613\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"\"},\"finish_reason\":null}]}\n\ndata: {\"id\":\"chatcmpl-7YK1bd5RqjEmR7W5TQn3hAqoyA0Zy\",\"object\":\"chat.completion.chunk\",\"created\":1688414483,\"model\":\"gpt-3.5-turbo-0613\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"OK\"},\"finish_reason\":null}]}\n\n";
    //     let result = parse_byte_to_response_dto(input);
    //     assert!(result.is_ok())
    // }
    //
}
