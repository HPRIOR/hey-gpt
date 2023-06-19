mod conversation;
mod gpt_context;
mod gpt_request;
mod output;
mod user;

use std::{error::Error, pin::Pin};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::Stream;
use reqwest::Client;

use crate::model::{Algo, Model};

use self::{
    conversation::YamlHistory, gpt_context::GptContext, gpt_request::GptRequest, output::Output,
    user::User,
};

//--- Shared ---//
#[derive(Debug)]
pub struct QueryWindow {
    pub min: Option<DateTime<Utc>>,
    pub max: Option<DateTime<Utc>>,
}

//--- Ai Requests ---//
pub struct EditRequestInput {
    pub instruction: String,
    pub input: String,
}

#[derive(Debug)]
pub struct ChatRequestInput {
    pub role: String,
    pub content: String,
}

#[async_trait]
pub trait RequestEffect: Sync + Send {
    async fn chat_request_stream(
        &self,
        request: &[ChatRequestInput],
        model: &Model,
    ) -> Result<Pin<Box<dyn Stream<Item = Vec<String>> + Send + 'static>>, Box<dyn Error>>;

    async fn edit_request_stream(
        &self,
        request: EditRequestInput,
        algo: &Algo,
    ) -> Result<Pin<Box<dyn Stream<Item = Vec<String>> + Send + 'static>>, Box<dyn Error>>;
}

//--- Display Output ---//
#[async_trait]
pub trait DisplayEffect: Send + Sync {
    async fn print_stream(
        &self,
        mut input: Pin<Box<dyn Stream<Item = Vec<String>> + Send>>,
    ) -> Vec<String>;
    fn print(&self, input: &str);
    fn eprint(&self, input: &str);
}

//--- User Input ---//
pub enum UserCycleResponse {
    Accept,
    Edit,
    NextRight,
    NextLeft,
}

pub trait UserEffect: Send + Sync {
    fn elicit_cycle_response(&self, user_prompt: &str) -> UserCycleResponse;
    fn edit_data_gen_prompt(&self, initial_prompt: &str) -> Result<String, Box<dyn Error>>;
}

//--- Memory ---//

pub struct MemSaveInp {
    pub text: String,
    pub author: String,
}

#[derive(Debug)]
pub struct MemQueryOpt {
    pub category: String,
    pub query_window: QueryWindow,
}

#[derive(Debug)]
pub struct MemOutput {
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub author: String,
    pub category: String,
}

#[async_trait]
pub trait AiMemory: Sync + Send {
    async fn save(
        &self,
        ctx_inp: &[MemSaveInp],
        category: &str,
    ) -> Result<Vec<String>, Box<dyn Error>>;
    async fn query(
        &self,
        query: &str,
        query_opts: &[MemQueryOpt],
    ) -> Result<Vec<MemOutput>, Box<dyn Error>>;
    async fn delete(&self, id: &str) -> Result<(), Box<dyn Error>>;
}

//--- Conversation History ---//

pub struct SaveHistoryInput {
    pub author: String,
    pub content: String,
}

#[derive(Debug)]
pub struct HistoryOutput {
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub content: String,
}

#[async_trait]
pub trait HistoryEffect: Sync + Send {
    async fn save_history(
        &self,
        input: &[SaveHistoryInput],
    ) -> Result<(), Box<dyn Error>>;
    async fn get_history(
        &self,
        len: usize,
    ) -> Result<Vec<HistoryOutput>, Box<dyn Error>>;
}

pub struct Effects {
    pub requester: Box<dyn RequestEffect>,
    pub displayer: Box<dyn DisplayEffect>,
    pub user: Box<dyn UserEffect>,
    pub context: Box<dyn AiMemory>,
    pub history: Box<dyn HistoryEffect>,
}

impl Effects {
    pub fn new(model: &Model) -> Self {
        let requester = Box::new(GptRequest::new(Client::new(), model.open_ai_token.clone()));

        let displayer = Box::new(Output);
        let user_displayer = Box::new(Output);
        let user = Box::new(User(user_displayer));

        let context = Box::new(GptContext::new(
            Client::new(),
            model.context_token.clone(),
            model.memory.top_k,
            model.config.context_url.to_string()
        ));
        let history = Box::new(YamlHistory::new(&model.memory.convo_path));

        Self {
            requester,
            displayer,
            user,
            context,
            history,
        }
    }
}
