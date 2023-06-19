use std::{
    error::Error,
    process::exit,
};

use async_recursion::async_recursion;
use clap::Parser;
use data::model;
use effect::Effects;
use log::debug;
use model::Model;
use states::Action;

use crate::{states::init::InitState, data::args::CliArgs};


mod output;
mod utils;
mod effect;
mod states;
mod data;

// todo, make configurable 
pub const COMPLETION_URL: &str = "https://api.openai.com/v1/chat/completions";
pub const EDIT_URL: &str = "https://api.openai.com/v1/edits";
// pub const CONTEXT_URL: &str = "http://localhost:8000"; 
pub const DEFAULT_CONVO: &str = "a4c80afe-f225-11ed-a05b-0242ac120003"; 

fn setup_logger(is_debug: bool) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} - {}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(if is_debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    setup_logger(args.debug)?;

    debug!("Converting arguments to domain model");
    let model = args.into_domain()?;

    debug!("composing effects");
    let effects = Effects::new(&model);

    debug!("Entering main loop");
    main_loop(Box::new(InitState(effects)), model).await;
    Ok(())
}

#[async_recursion(?Send)]
async fn main_loop(action: Box<dyn Action>, model: Model) {
    debug!("Executing action for {} state", action._type());
    debug!("Using model {:#?}", &model);

    let execute_result = action.execute(model).await;

    match execute_result {
        Ok((new_state, new_model)) => {
            main_loop(new_state, new_model).await;
        }
        Err(error) => {
            eprint!("An error has occured: {}", error);
            exit(1)
        }
    }
}
