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

use crate::model::Model;

use self::{
    conversation::YamlHistory, gpt_context::LongTermGptMemory, gpt_request::GptRequest, output::Output,
    user::User,
};


//--- Ai Requests ---//
#[derive(Debug)]
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
pub trait AiRequestEffect: Sync + Send {
    async fn chat_request_stream(
        &self,
        request: &[ChatRequestInput],
    ) -> Result<Pin<Box<dyn Stream<Item = Vec<String>> + Send + 'static>>, Box<dyn Error>>;

    async fn edit_request_stream(
        &self,
        request: EditRequestInput,
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

pub trait InteractionEffect: Send + Sync {
    fn elicit_cycle_response(&self, user_prompt: &str) -> UserCycleResponse;
    fn edit_data_gen_prompt(&self, initial_prompt: &str) -> Result<String, Box<dyn Error>>;
}

//--- Memory ---//
#[derive(Debug)]
pub struct QueryWindow {
    pub min: Option<DateTime<Utc>>,
    pub max: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct LongMemSaveInp {
    pub text: String,
    pub author: String,
}

#[derive(Debug)]
pub struct LongMemQueryOpt {
    pub category: String,
    pub query_window: QueryWindow,
}

#[derive(Debug)]
pub struct LongMemOutput {
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub author: String,
    pub category: String,
}

#[async_trait]
pub trait LongMemEffect: Sync + Send {
    async fn save(
        &self,
        input: &[LongMemSaveInp],
        category: &str,
    ) -> Result<Vec<String>, Box<dyn Error>>;
    async fn query(
        &self,
        query: &str,
        query_opts: &[LongMemQueryOpt],
    ) -> Result<Vec<LongMemOutput>, Box<dyn Error>>;
    async fn delete(&self, id: &str) -> Result<(), Box<dyn Error>>;
}

pub struct ShortMemInput {
    pub author: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ShortMemOutput {
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub content: String,
}

#[async_trait]
pub trait ShortMemEffect: Sync + Send {
    async fn save_history(
        &self,
        input: &[ShortMemInput],
    ) -> Result<(), Box<dyn Error>>;
    async fn get_history(
        &self,
        len: usize,
    ) -> Result<Vec<ShortMemOutput>, Box<dyn Error>>;
}

pub struct Effects {
    pub requester: Box<dyn AiRequestEffect>,
    pub displayer: Box<dyn DisplayEffect>,
    pub user: Box<dyn InteractionEffect>,
    pub context: Box<dyn LongMemEffect>,
    pub history: Box<dyn ShortMemEffect>,
}

impl Effects {
    pub fn new(model: &Model) -> Self {
        let requester = Box::new(GptRequest::new(Client::new(), model.open_ai_token.clone(), model.clone()));

        let displayer = Box::new(Output);
        let user_displayer = Box::new(Output);
        let user = Box::new(User(user_displayer));

        let context = Box::new(LongTermGptMemory::new(
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
