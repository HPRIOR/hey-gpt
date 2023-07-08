use std::error::Error;

use async_trait::async_trait;

use crate::{
    effect::{ChatRequestInput, Effects},
    model::{Model, Prompt},
};

use super::{preview::PreviewState, request::RequestState, Action};

pub struct DataRequestState {
    pub effects: Effects,
    pub prompt: String,
}

#[async_trait]
impl Action for DataRequestState {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>> {
        let request_input = ChatRequestInput {
            role: "user".to_string(),
            content: self.prompt.clone(),
        };

        let request = self
            .effects
            .requester
            .chat_request_stream(&[request_input])
            .await?;

        let data = self.effects.displayer.print_stream(request).await;

        let preview_wanted = &model.config.preview_data_generation;
        if *preview_wanted {
            Ok((
                Box::new(PreviewState {
                    effects: self.effects,
                    preview_data: data,
                    preview_index: 0,
                    prompt: self.prompt,
                    should_display: false, // already printed here for first run
                }),
                model,
            ))
        } else {
            Ok((
                Box::new(RequestState(self.effects)),
                Model {
                    prompt: Prompt {
                        generated_data: Some(data[0].clone()),
                        ..model.prompt
                    },
                    ..model
                },
            ))
        }
    }

    fn _type(&self) -> String {
        String::from("Data Request")
    }
}
