pub mod init;
mod data_request;
mod preview;
mod request;
mod success;

use std::error::Error;

use async_trait::async_trait;

use crate::model::Model;

#[async_trait]
pub trait Action {
    async fn execute(
        self: Box<Self>,
        model: Model,
    ) -> Result<(Box<dyn Action>, Model), Box<dyn Error>>;
    fn _type(&self) -> String;
}
