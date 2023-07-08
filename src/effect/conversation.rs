use std::error::Error;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::utils;

use super::{ShortMemEffect, ShortMemInput, ShortMemOutput};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DialogueSegment {
    role: String,
    content: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Script {
    dialogue: Vec<DialogueSegment>,
}

pub struct YamlHistory {
    convo_path: String,
}

impl YamlHistory {
    pub fn new(convo_path: &str) -> Self {
        Self {
            convo_path: convo_path.to_string(),
        }
    }
}

#[async_trait]
impl ShortMemEffect for YamlHistory {
    async fn save_history(&self, input: &[ShortMemInput]) -> Result<(), Box<dyn Error>> {
        if utils::file_exists_async(&self.convo_path).await {
            let script: Script = utils::deserialise_from_file_async(&self.convo_path).await?;

            let mut new_dialogue_segment: Vec<DialogueSegment> = input
                .iter()
                .map(|ShortMemInput { author, content }| DialogueSegment {
                    role: author.clone(),
                    content: content.clone(),
                    created_at: Utc::now(),
                })
                .collect();

            let mut dialogue = script.dialogue;
            dialogue.append(&mut new_dialogue_segment);
            utils::write_to_async(&self.convo_path, &Script { dialogue }).await?;
        } else {
            let dialogue: Vec<DialogueSegment> = input
                .iter()
                .map(|ShortMemInput { author, content }| DialogueSegment {
                    role: author.clone(),
                    content: content.clone(),
                    created_at: Utc::now(),
                })
                .collect();
            utils::write_to_async(&self.convo_path, &Script { dialogue }).await?;
        };
        Ok(())
    }

    async fn get_history(&self, len: usize) -> Result<Vec<ShortMemOutput>, Box<dyn Error>> {
        let script: Script = utils::deserialise_from_file_async(&self.convo_path).await?;

        let dialogue_window = {
            let dialogue_len = script.dialogue.len();
            let difference = dialogue_len.checked_sub(len);
            &script.dialogue[difference.unwrap_or(0)..dialogue_len]
        };

        Ok(dialogue_window
            .iter()
            .map(
                |DialogueSegment {
                     role,
                     content,
                     created_at,
                 }| ShortMemOutput {
                    author: role.to_string(),
                    content: content.to_string(),
                    created_at: *created_at,
                },
            )
            .collect())
    }
}
