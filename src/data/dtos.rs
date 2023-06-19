use std::{error::Error, fmt::Display};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EditRequestDTO {
    pub model: String,
    pub input: String,
    pub instruction: String,
    pub n: i32,
    pub temperature: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequestMsgDTO {
    pub content: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequestDTO {
    pub messages: Vec<ChatRequestMsgDTO>,
    pub model: String,
    pub n: i32,
    pub temperature: f32,
    pub max_tokens: Option<i32>,
    pub stream: bool,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UsageDTO {
    pub completion_tokens: i32,
    pub prompt_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponseMsgDTO {
    pub content: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatChoiceDTO {
    pub finish_reason: String,
    pub index: i32,
    pub message: ChatResponseMsgDTO,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponseDTO {
    pub choices: Vec<ChatChoiceDTO>,
    pub created: i32,
    pub id: String,
    pub model: String,
    pub object: String,
    pub usage: UsageDTO,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamChatResponseMsgDTO {
    pub content: Option<String>,
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamChatChoiceDTO {
    pub finish_reason: Option<String>,
    pub index: i32,
    pub delta: StreamChatResponseMsgDTO,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamChatResponseDTO {
    pub choices: Vec<StreamChatChoiceDTO>,
    pub created: i32,
    pub id: String,
    pub model: String,
    pub object: String,
}

#[derive(Debug)]
struct StreamChatResponseDeserialiseError(Vec<String>);
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

impl TryFrom<Bytes> for StreamChatResponseDTO {
    type Error = Box<dyn Error>;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let data_str = String::from_utf8(value.to_vec())?;

        let without_data = data_str.replace("data: ", "");

        let split: Vec<&str> = without_data
            .split("\n\n")
            .filter(|s| !s.is_empty() && !s.contains("DONE") && s.contains("content"))
            .collect();

        if let Some(s) = split.first() {
            Ok(serde_json::from_str(s)?)
        } else {
            Err(Box::new(StreamChatResponseDeserialiseError(
                split.iter().map(|s| s.to_string()).collect(),
            )))
        }
    }
}

impl From<StreamChatResponseDTO> for Vec<String> {
    fn from(value: StreamChatResponseDTO) -> Self {
        value
            .choices
            .into_iter()
            .filter_map(|choice| choice.delta.content)
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditChoiceDTO {
    pub index: i32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditResponseDTO {
    pub choices: Vec<EditChoiceDTO>,
    pub created: i32,
    pub object: String,
    pub usage: UsageDTO,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpsertMetadataDTO {
    pub created_at: String,
    pub source_id: String,
    pub source: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpsertResponseDTO {
    pub ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetrievalUpsertDTO {
    pub id: Option<String>,
    pub metadata: Option<UpsertMetadataDTO>,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetreivalUpsertWrapperDTO {
    pub documents: Vec<RetrievalUpsertDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetrievalDeleteDTO {
    pub ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetrievalFilterDTO {
    pub source_id: Option<String>,
    pub source: String,
    pub author: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryDTO {
    pub query: String,
    pub filter: Option<RetrievalFilterDTO>,
    pub top_k: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RetrievalQueryDTO {
    pub queries: Vec<QueryDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentMetaDataDTO {
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub url: Option<String>,
    pub created_at: Option<String>,
    pub author: Option<String>,
    pub document_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentResultDTO {
    pub id: String,
    pub text: String,
    pub metadata: DocumentMetaDataDTO,
    pub embedding: Vec<f64>,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResultDTO {
    pub query: String,
    pub results: Vec<DocumentResultDTO>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultWrapperDTO {
    pub results: Vec<SearchResultDTO>,
}
