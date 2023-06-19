use std::{error::Error, pin::Pin};

use crate::data::dtos::{
    ChatRequestDTO, ChatRequestMsgDTO, EditRequestDTO, EditResponseDTO, StreamChatResponseDTO,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use log::debug;
use reqwest::Client;

use crate::data::model::Model;
use crate::{model::Algo, utils::Transpose, COMPLETION_URL, EDIT_URL};

use super::{ChatRequestInput, EditRequestInput, RequestEffect};

pub struct GptRequest {
    client: Client,
    auth_token: String,
}

impl GptRequest {
    pub fn new(client: Client, auth_token: String) -> Self {
        Self { client, auth_token }
    }
}

#[async_trait]
impl RequestEffect for GptRequest {
    async fn chat_request_stream(
        &self,
        request: &[ChatRequestInput],
        model: &Model,
    ) -> Result<Pin<Box<dyn Stream<Item = Vec<String>> + Send + 'static>>, Box<dyn Error>> {
        let request = ChatRequestDTO {
            messages: request
                .iter()
                .map(|ChatRequestInput { role, content }| ChatRequestMsgDTO {
                    content: content.to_string(),
                    role: role.to_string(),
                })
                .collect(),
            model: model.algo.chat_model.to_string(),
            n: 1,
            temperature: model.algo.temp,
            max_tokens: model.algo.max_tokens,
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

        let result = response
            .error_for_status()?
            .bytes_stream()
            .map(
                |byte_result| -> Result<StreamChatResponseDTO, Box<dyn Error>> {
                    match byte_result {
                        Ok(byte) => {
                            let dto_result: Result<StreamChatResponseDTO, Box<dyn Error>> =
                                byte.try_into();
                            dto_result
                        }
                        Err(e) => Err(Box::new(e)),
                    }
                },
            )
            .map(|dto_result| -> Vec<String> {
                dto_result.map(|dto| dto.into()).unwrap_or_else(|e| {
                    debug!("Deserialising problem: {}", e);
                    vec![]
                })
            });

        Ok(Box::pin(result))
    }

    async fn edit_request_stream(
        &self,
        request: EditRequestInput,
        algo: &Algo,
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
                model: algo.edit_model.clone(),
                n: 1,
                temperature: algo.temp,
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
