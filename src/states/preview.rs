use std::error::Error;

use async_trait::async_trait;

use crate::{
    effect::{Effects, UserCycleResponse},
    model::{Model, Prompt},
};

use super::{data_request::DataRequestState, request::RequestState, Action};

pub struct PreviewState {
    pub effects: Effects,
    pub preview_data: Vec<String>,
    pub preview_index: usize,
    pub prompt: String,
    pub should_display: bool,
}

#[async_trait]
impl Action for PreviewState {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>> {
        let maybe_preview = self.preview_data.get(self.preview_index);
        match maybe_preview {
            Some(preview) => {
                if self.should_display {
                    self.effects.displayer.print(preview);
                }

                let cycle_msg = if self.preview_data.len() > 1 {
                    "\n[Press enter to continue; e to Edit; Cycle with <-/->/h/l]"
                } else {
                    "\n[Press enter to continue; e to Edit]"
                };

                let user_response = self.effects.user.elicit_cycle_response(cycle_msg);

                match user_response {
                    UserCycleResponse::NextRight => Ok((
                        Box::new(PreviewState {
                            preview_index: self.preview_index + 1,
                            should_display: true,
                            ..*self
                        }),
                        model,
                    )),
                    UserCycleResponse::NextLeft => Ok((
                        Box::new(PreviewState {
                            preview_index: self.preview_index - 1,
                            should_display: true,
                            ..*self
                        }),
                        model,
                    )),
                    UserCycleResponse::Edit => {
                        let new_prompt = self.effects.user.edit_data_gen_prompt(&self.prompt)?;
                        Ok((
                            Box::new(DataRequestState {
                                effects: self.effects,
                                prompt: new_prompt,
                            }),
                            model,
                        ))
                    }
                    UserCycleResponse::Accept => Ok((
                        Box::new(RequestState(self.effects)),
                        Model {
                            prompt: Prompt {
                                generated_data: Some(preview.clone()),
                                ..model.prompt
                            },
                            ..model
                        },
                    )),
                }
            }
            // index out of range
            None => {
                let new_index = if self.preview_index >= self.preview_data.len() {
                    0
                } else {
                    self.preview_data.len() - 1
                };

                Ok((
                    Box::new(PreviewState {
                        preview_index: new_index,
                        should_display: true,
                        ..*self
                    }),
                    model,
                ))
            }
        }
    }

    fn _type(&self) -> String {
        String::from("Preview Generated Data")
    }
}
