use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub struct Algo {
    pub chat_model: String,
    pub edit_model: String,
    pub temp: f32,
    pub max_tokens: Option<i32>,
}


#[derive(Debug, Clone)]
pub enum EditData {
    DataFromPrompt(String),
    DataFromStdIn(String),
}

#[derive(Debug, Clone)]
pub enum ChatData {
    DataFromPrompt(String),
    DataFromStdIn(String),
    NoAdditionalData,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Chat(ChatData),
    Edit(EditData),
}

#[derive(Debug, Clone, Default)]
pub struct Memory {
    pub top_k: u32,
    pub memories: Vec<String>,
    pub convo: String,
    pub convo_len: usize,
    pub enabled: bool,
    pub convo_path: String
}

#[derive(Debug, Clone, Default)]
pub struct Output {
    pub failure_msg: Option<String>,
    pub chat_results: Option<Vec<String>>,
    pub edit_results: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub debug: bool,
    pub preview_data_generation: bool,
    pub context_url: String
}


// prompt could maybe be simplified 
#[derive(Debug, Clone, Default)]
pub struct Prompt {
    pub generated_data: Option<String>,
    pub prompt: String,
    pub final_chat_prompt: Option<String>,
    pub act_as: String, 
}

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub algo: Algo,
    pub config: Config,
    pub mode: Mode,
    pub output: Output,
    pub prompt: Prompt,
    pub memory: Memory,
    pub open_ai_token: String,
    pub context_token: String,
}

impl Model {
    pub fn with_chat_prompt(self, prompt: String) -> Model {
        Model {
            prompt: Prompt {
                final_chat_prompt: Some(prompt),
                ..self.prompt
            },
            ..self
        }
    }

    pub fn with_chat_response(self, response: Vec<String>) -> Model {
        Model {
            output: Output {
                chat_results: Some(response),
                ..self.output
            },
            ..self
        }
    }
    pub fn with_edit_response(self, response: Vec<String>) -> Model {
        Model {
            output: Output {
                edit_results: Some(response),
                ..self.output
            },
            ..self
        }
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Chat(ChatData::NoAdditionalData)
    }
}

