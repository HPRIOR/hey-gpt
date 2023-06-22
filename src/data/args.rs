use std::{
    env,
    error::Error,
    io::{self, BufRead, Write},
    path::PathBuf,
    process::exit,
};

use clap::{command, Parser};
use log::debug;

use crate::{
    utils::{self, file_exists},
    DEFAULT_CONVO,
};

use super::model::{Algo, ChatData, Config, EditData, Memory, Mode, Model, Output, Prompt};

fn default_act_as() -> String {
    "You are a helpful AI assistant that will give responses in a computer terminal".to_string()
}

#[derive(Parser, Debug)]
#[command(author = "Harry Prior")]
#[command(version = "0.1.0")]
#[command(about = "CLI wrapper around chat-gpt cli")]
#[command(long_about=None)]
pub struct CliArgs {
    /// The prompt to give to the model
    pub prompt: String,

    /// Chat model
    #[arg(short, long)]
    pub chat_model: Option<String>,

    /// Edit model
    #[arg(long)]
    pub edit_model: Option<String>,

    /// Maximum number of tokens to generate in a completion or edit
    #[arg(long)]
    pub max_tokens: Option<i32>,

    /// The tempurature of the model
    #[arg(short, long)]
    pub temp: Option<f32>,

    /// Seed initial prompt with data produced by this prompt. Previews generated data unless no_preview option used
    #[arg(short, long)]
    pub data_prompt: Option<String>,

    /// Do not prompt user for previews
    #[arg(short = 'y', long)]
    pub no_preview: Option<bool>,

    /// Specify environment variable storing openai auth token
    #[arg(long)]
    pub open_ai_token_env: Option<String>,

    /// Specify token to use for api auth
    #[arg(long)]
    pub open_ai_token: Option<String>,

    /// Specify environment variable storing context auth token
    #[arg(long)]
    pub context_ai_token_env: Option<String>,

    /// Token to use for api auth
    #[arg(long)]
    pub context_ai_token: Option<String>,

    /// Store and retreive short term convo history
    #[arg(long)]
    pub convo: Option<String>,

    /// Store and retreive short term convo history
    #[arg(long)]
    pub convo_length: Option<usize>,

    /// Directory containing short term conversation memory files
    #[arg(long)]
    pub convo_dir: Option<String>,

    /// Add custom message for system
    #[arg(long)]
    pub act_as: Option<String>,

    /// The number of items to retrieve from vector database when using context
    #[arg(long)]
    pub top_k: Option<u32>,

    /// retrieve memories for use in query
    #[arg(long)]
    pub memories: Option<Vec<String>>,

    /// retrieve memories for use in query
    #[arg(long)]
    pub memory: Option<bool>,

    /// Print debug output
    #[arg(long)]
    pub debug: bool,

    /// Edit request, data must be supplied through std-in or --data argument
    #[arg(long, short)]
    pub edit: bool,

    #[arg(long)]
    /// Url of context database
    pub context_url: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct ConfigArgs {
    pub chat_model: Option<String>,
    pub edit_model: Option<String>,
    pub max_tokens: Option<i32>,
    pub temp: Option<f32>,
    pub no_preview: Option<bool>,
    pub open_ai_token_env: Option<String>,
    pub open_ai_token: Option<String>,
    pub context_ai_token_env: Option<String>,
    pub context_ai_token: Option<String>,
    pub convo: Option<String>,
    pub convo_length: Option<usize>,
    pub convo_dir: Option<String>,
    pub act_as: Option<String>,
    pub top_k: Option<u32>,
    pub memories: Option<Vec<String>>,
    pub memory: Option<bool>,
    pub always_edit: Option<bool>,
    pub context_url: Option<String>,
}

fn get_stdin() -> String {
    if !atty::is(atty::Stream::Stdin) {
        let stdin = io::stdin();
        let mut buffer = String::new();
        for line in stdin.lock().lines().flatten() {
            buffer.push_str(line.as_str());
        }

        buffer
    } else {
        Default::default()
    }
}

impl CliArgs {
    pub fn into_domain(self) -> Result<Model, Box<dyn Error>> {
        // this probably belongs elsewhere
        let config_args = {
            let home = env::var("HOME")?;
            let paths = vec![
                format!("{}/.config/hey_gpt/config", home),
                format!("{}/hey_gpt/config", home),
            ];

            debug!(
                "Looking for configuration files in the following order: {:#?}",
                paths
            );

            paths
                .iter()
                .find(|path| utils::file_exists(path))
                .map(|existing_file_path| {
                    utils::deserialise_from_file::<ConfigArgs>(existing_file_path)
                        .unwrap_or_default()
                })
                .unwrap_or_default()
        };

        debug!("Found the following configuration: {:#?}", config_args);

        debug!("Building model");
        let open_ai_token = self.open_ai_token.unwrap_or_else(|| {
            config_args.open_ai_token.unwrap_or_else(|| {
                match std::env::var(
                    self.open_ai_token_env.unwrap_or(
                        config_args
                            .open_ai_token_env
                            .unwrap_or("OPENAI_KEY".to_string()),
                    ),
                ) {
                    Ok(token) => token,
                    _ => panic!(
                    "Could not find openai token in environment and it was not provided by user"
                ),
                }
            })
        });

        let user_wants_memory = self.memory.unwrap_or(config_args.memory.unwrap_or(false));

        // only required if user wants persistant memory
        let context_token = if user_wants_memory {
            self.context_ai_token.unwrap_or_else(|| {
                config_args.context_ai_token
                    .unwrap_or_else(|| match std::env::var(self.context_ai_token_env.unwrap_or(config_args.context_ai_token_env.unwrap_or("AI_CONTEXT_KEY".to_string()))) {
                        Ok(token) => token,
                        _ => panic!(
                            "Could not find ai context token in environment and it was not provided by user"
                        ),
                    })
            })
        } else {
            Default::default()
        };

        let mode = {
            // for now favour data argument over stdin by default
            let stdin = get_stdin();
            let data_prompt = self.data_prompt.clone();

            if config_args.always_edit.unwrap_or(self.edit) {
                let edit_data = match (data_prompt, stdin.is_empty()) {
                    (Some(data_prompt), true) | (Some(data_prompt), false) => {
                        EditData::DataFromPrompt(data_prompt)
                    }
                    (None, false) => EditData::DataFromStdIn(stdin),
                    _ => {
                        eprintln!("Data must be provided for edit mode");
                        exit(1);
                    }
                };
                Mode::Edit(edit_data)
            } else {
                let chat_data = match (data_prompt, stdin.is_empty()) {
                    (Some(data), true) | (Some(data), false) => ChatData::DataFromPrompt(data),
                    (None, false) => ChatData::DataFromStdIn(stdin),
                    _ => ChatData::NoAdditionalData,
                };
                Mode::Chat(chat_data)
            }
        };

        debug!("Mode: {:#?}", mode);

        let algo = Algo {
            chat_model: self.chat_model.unwrap_or(
                config_args
                    .chat_model
                    .unwrap_or("gpt-3.5-turbo".to_string()),
            ),
            edit_model: self.edit_model.unwrap_or(
                config_args
                    .edit_model
                    .unwrap_or("text-davinci-edit-001".to_string()),
            ),
            temp: self.temp.unwrap_or(config_args.temp.unwrap_or(0.7)),
            max_tokens: self.max_tokens.map_or_else(|| config_args.max_tokens, Some),
        };

        debug!("Algo: {:#?}", algo);

        let config = Config {
            debug: self.debug,
            preview_data_generation: self
                .data_prompt
                .map(|_| {
                    !self
                        .no_preview
                        .unwrap_or(config_args.no_preview.unwrap_or(false))
                })
                .unwrap_or(false),
            context_url: self.context_url.unwrap_or_else(|| {
                if user_wants_memory {
                    config_args
                        .context_url
                        .expect("Context url should be set if memory enabled")
                } else {
                    String::new()
                }
            }),
        };

        debug!("Config: {:#?}", config);

        let output = Output {
            failure_msg: None,
            chat_results: None,
            edit_results: None,
        };

        debug!("Output: {:#?}", output);

        let prompt = Prompt {
            generated_data: None,
            prompt: self.prompt,
            final_chat_prompt: None,
            act_as: self
                .act_as
                .unwrap_or(config_args.act_as.unwrap_or(default_act_as())),
        };

        debug!("Prompt: {:#?}", prompt);

        let convo_file_path: PathBuf = vec![
            self.convo_dir
                .clone()
                .map(|dir| {
                    dir.replace(
                        "$HOME",
                        env::var("HOME")
                            .expect("Could not find HOME environment variable")
                            .as_str(),
                    )
                })
                .unwrap_or_else(|| {
                    config_args
                        .convo_dir
                        .clone()
                        .expect("Directory for storing conversation should be set")
                }),
            self.convo.clone().unwrap_or(
                config_args
                    .convo
                    .clone()
                    .unwrap_or(DEFAULT_CONVO.to_string()),
            ),
        ]
        .into_iter()
        .collect();

        if !file_exists(convo_file_path.to_str().unwrap()) {
            match std::fs::File::create(&convo_file_path) {
                Ok(mut file) => {
                    let str = "dialogue: ";
                    file.write_all(str.as_bytes()).unwrap();
                }
                Err(e) => panic!(
                    "Could not create conversation file at '{}': {}",
                    convo_file_path.to_str().unwrap(),
                    e
                ),
            }
        }

        let memory = Memory {
            top_k: self.top_k.unwrap_or(config_args.top_k.unwrap_or(3)),
            memories: self
                .memories
                .unwrap_or(config_args.memories.unwrap_or(vec![])),
            enabled: user_wants_memory,
            convo: self
                .convo
                .unwrap_or(config_args.convo.unwrap_or(DEFAULT_CONVO.to_string())),
            convo_len: self
                .convo_length
                .unwrap_or(config_args.convo_length.unwrap_or(0)),
            convo_path: convo_file_path.to_str().unwrap().to_owned(),
        };

        debug!("Memory: {:#?}", memory);

        Ok(Model {
            algo,
            config,
            mode,
            output,
            prompt,
            memory,
            open_ai_token,
            context_token,
        })
    }
}
