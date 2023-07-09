# Hey-gpt

A command line interface for chat-gpt

---

## Examples

### Simple query
```bash
hey-gpt 'Help me with something'
```

### Pipe data to query
```bash
hey-gpt --help | hey-gpt 'Summarise this application 50 words or less'
```

### Generate data for query
```bash
hey-gpt 'Sort the input data from most to least incompetent' --data 'Generate a list of every prime minister of the UK' --no-preview 
```

> Without the `--no-preview` argument, user will be asked to accept, retry or edit data prompt

### Store and retrieve conversation history 

```bash
hey-gpt 'This is a great discussion, im glad you will remember it forever' --convo 'great-discussion' 
```
Conversation history will be stored in a yaml file so that it can be included in future queries. To limit the length of the conversation history use the `--convo-length` argument:

```bash
hey-gpt 'This is a great discussion, im glad you will remember at least 3 interactions back' --convo 'great-discussion' --convo-length 3
```

### Store and retrieve long-term conversation history

```bash
hey-gpt 'Can you remind me what you were saying about nuclear war a few months ago?' --convo 'armageddon-likelihood' --memory

```

Conversation history will be stored in a vector database. Database will also be queried to provide relevant context from conversation history. The database will only be queried beyond what can be retrieved by short-term memory. `--top-k` argument specifies the number of items to return from vector database. Full example:


```bash
hey-gpt 'Can you remind me what you were saying about nuclear war a few months ago?' --convo 'armageddon-likelihood' --convo-length 3 --memory --top-k 5
```
This retrieves 3 question and responses from the 'armageddon-likelihood' conversation. The outgoing query will then be sent along with the 5 top matching text items from entire saved conversation history in the vector database. 
The response to this query is also saved for later retrieval in both short-term yaml memory and the long-term vector database.

---
## Installation

### Build

```bash
cargo build --release --out-dir "/path/to/dir/in/$PATH"
```

### Config

Add a configuration file at `$HOME/.config/hey_gpt/config.yaml` or `$HOME/hey_gpt/config.yaml`.

It can contain the following arguments:

``` yaml
chat_model: String
edit_model: String
max_tokens: i32
temp: f32
open_ai_token_env: String
open_ai_token: String
retrieval_api_bearer_env: String
retrieval_api_bearer: String
convo: String
convo_length: usize
convo_dir: String
act_as: String
top_k: u32
memories: Vec<String,
retrieval_plugin_url: String
```

#### Manditory configs

`convo_dir` must be set here or in the command arguments, and is the location where conversations will be stored for short term memory.

`open_ai_token` and `retrieval_plugin_url` can be set through configuration, command arguments, or through the environment variable specified in `open_ai_token_env` and `retrieval_plugin_url_env` or the env variable `OPENAI_KEY` and `RETRIEVAL_API_BEARER` respectively.


### Long term memory

Long term memory has two dependencies. The ChatGPT retrieval plugin (https://github.com/openai/chatgpt-retrieval-plugin), and a vector database. A docker-compose compose file can be found at the root of the project which will quickly spin up these dependencies:

```bash
HOST_PERSIST_DIR="$HOME/path/to/persistant/storage/directory" docker-compose up -d
```

The environment variable `HOST_PERSIST_DIR` must be provided and sets a volume mount which the vector database uses for storage.

`OPENAI_KEY` and `RETRIEVAL_API_BEARER` must also be set in the environment from the 'Mandatory configs' section

                                                                                                                 

















