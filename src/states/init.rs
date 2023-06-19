use std::error::Error;

use async_trait::async_trait;

use crate::{effect::Effects, model::{Model, Mode, ChatData, EditData}};

use super::{Action, data_request::DataRequestState, request::RequestState};


pub struct InitState(pub Effects);

#[async_trait]
impl Action for InitState {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>> {
        match &model.mode {
            Mode::Chat(ChatData::DataPrompt(data_request_prompt))
            | Mode::Edit(EditData::DataPrompt(data_request_prompt)) => Ok((
                Box::new(DataRequestState {
                    effects: (*self).0,
                    prompt: data_request_prompt.clone(),
                }),
                model,
            )),
            _ => Ok((Box::new(RequestState((*self).0)), model)),
        }
    }

    fn _type(&self) -> String {
        String::from("Init")
    }
}
