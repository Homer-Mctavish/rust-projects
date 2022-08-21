use lambda_runtime::Error;
use lambda_runtime::Context;
use serde_derive::{Serialize, Deserialize};



#[derive(Deserialize)]
struct Event {
    username: String,
}

#[derive(Serialize)]
struct Output {
    message: String,
}

async fn handler(event: Event, context: Context) -> Result<Output, Error> {
    let message = format!("Welcome, {}!", event.username);
    Ok(Output {
        message: message,
    })
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = lambda_runtime::handler_fn(handler);
    lambda_runtime::run(handler).await?;
    Ok(())
}