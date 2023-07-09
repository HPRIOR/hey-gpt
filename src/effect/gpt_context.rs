use std::error::Error;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::debug;
use reqwest::Client;

use crate::data::dtos::{
    DocumentResultDTO, QueryDTO, ResultWrapperDTO, RetreivalUpsertWrapperDTO, RetrievalDeleteDTO,
    RetrievalFilterDTO, RetrievalQueryDTO, RetrievalUpsertDTO, UpsertMetadataDTO,
    UpsertResponseDTO,
};

use super::{LongMemEffect, LongMemOutput, LongMemQueryOpt, LongMemSaveInp};

pub struct LongTermGptMemory {
    client: Client,
    bearer_auth: String,
    top_k: u32,
    context_url: String,
}

impl LongTermGptMemory {
    pub fn new(client: Client, bearer_auth: String, top_k: u32, context_url: String) -> Self {
        Self {
            client,
            bearer_auth,
            top_k,
            context_url,
        }
    }
}

#[async_trait]
impl LongMemEffect for LongTermGptMemory {
    async fn save(
        &self,
        ctx_input: &[LongMemSaveInp],
        category: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let documents: Vec<RetrievalUpsertDTO> = ctx_input
            .iter()
            .map(|LongMemSaveInp { text, author }| RetrievalUpsertDTO {
                id: None,
                metadata: Some(UpsertMetadataDTO {
                    created_at: chrono::offset::Utc::now().to_rfc3339(),
                    source_id: category.to_string(),
                    source: "email".to_string(),
                    author: author.to_string(),
                }),
                text: text.to_string(),
            })
            .collect();

        let upsert_wrapper = RetreivalUpsertWrapperDTO { documents };

        let post_result = self
            .client
            .post(format!("{}/upsert", self.context_url))
            .bearer_auth(&self.bearer_auth)
            .json(&upsert_wrapper)
            .send()
            .await?;

        let response = post_result
            .error_for_status()?
            .json::<UpsertResponseDTO>()
            .await?;

        Ok(response.ids)
    }

    async fn query(
        &self,
        query: &str,
        query_options: &[LongMemQueryOpt],
    ) -> Result<Vec<LongMemOutput>, Box<dyn Error>> {
        let queries: Vec<QueryDTO> = query_options
            .iter()
            .map(
                |LongMemQueryOpt {
                     category,
                     query_window,
                 }| QueryDTO {
                    query: query.to_string(),
                    top_k: self.top_k,
                    filter: Some(RetrievalFilterDTO {
                        source_id: Some(category.to_string()),
                        source: "email".to_string(),
                        author: None,
                        start_date: query_window.min.map(|min| min.to_rfc3339()),
                        end_date: query_window.max.map(|max| max.to_rfc3339()),
                    }),
                },
            )
            .collect();
        let query_wrapper = RetrievalQueryDTO { queries };

        let post_result = self
            .client
            .post(format!("{}/query", self.context_url))
            .bearer_auth(&self.bearer_auth) // todo get from env or config
            .json::<RetrievalQueryDTO>(&query_wrapper)
            .send()
            .await?;

        // dbg!(&post_result.text().await);
        let response = post_result
            .error_for_status()?
            .json::<ResultWrapperDTO>()
            .await?;

        let mut document_results: Vec<DocumentResultDTO> = response
            .results
            .into_iter()
            .flat_map(|search_result| search_result.results)
            .collect();

        document_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let top_k_best_match = document_results.iter().take(self.top_k as usize);

        let result: Vec<LongMemOutput> = top_k_best_match
            .into_iter()
            .map(|bm| LongMemOutput {
                text: bm.text.clone(),
                created_at: bm
                    .metadata
                    .created_at
                    .clone()
                    .map(|ca| ca.parse().unwrap())
                    .unwrap_or(DateTime::<Utc>::MIN_UTC),
                author: bm.metadata.author.clone().unwrap_or("".to_string()),
                category: bm.metadata.source_id.clone().unwrap_or("".to_string()),
            })
            .collect();

        Ok(result)
    }

    async fn delete(&self, id: &str) -> Result<(), Box<dyn Error>> {
        debug!("Deleting context with id: {}", id);
        let delete_ids = RetrievalDeleteDTO {
            ids: vec![id.to_string()],
        };

        let _ = self
            .client
            .delete(format!("{}/delete", self.context_url))
            .bearer_auth(&self.bearer_auth)
            .json(&delete_ids)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use reqwest::Client;

    use crate::effect::{
        gpt_context::LongTermGptMemory, LongMemEffect, LongMemQueryOpt, LongMemSaveInp, QueryWindow,
    };

    #[tokio::test]
    async fn test_hello_world() {
        let client = Client::new();
        let context  = 
            LongTermGptMemory::new(
                client, 
                "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkhhcnJ5IFByaW9yIiwiaWF0IjoxNTE2MjM5MDIyfQ.2Rf_63_CL4WMdmS01x8eQx7F4nbi7QXJW4QRAM2vl_0".to_string(), 
                10,
                "".to_string());

        let effect: Box<dyn LongMemEffect> = Box::new(context);

        let result = effect
            .save(
                &[LongMemSaveInp {
                    text: "This is a new embedding".to_string(),
                    author: "user".to_string(),
                }],
                "a4c80afe-f225-11ed-a05b-0242ac120003",
            )
            .await;

        println!("{:#?}", result);

        let query_opt = LongMemQueryOpt {
            category: "a4c80afe-f225-11ed-a05b-0242ac120003".to_string(),
            query_window: QueryWindow {
                min: None,
                max: Some(Utc::now()),
            },
        };
        let result = effect
            .query("can you summerise our conversation so far?", &[query_opt])
            .await;

        println!("{:#?}", result);
    }
}
