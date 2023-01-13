use anyhow::Result;
use lsp_client::{
    jsonrpc_types::{JsonRPCResult, Response},
    lsp_types::{request::Initialize, InitializeError, InitializeParams, InitializeResult},
    LspClient,
};
use serde::ser::Serialize;

pub async fn handle_request(client: &LspClient, request: &[&str]) {
    println!("got request: {:?}", request);

    let request_type = request.first().unwrap();
    let request_args = request.iter().skip(1).copied().collect::<Vec<_>>();

    let response = match *request_type {
        "initialize" => handle_initialize(client, request_args).await,
        other => {
            println!("Unknown request type: '{}'", other);
            return;
        }
    };

    handle_response(response);
}

pub async fn handle_initialize(
    client: &LspClient,
    args: Vec<&str>,
) -> Result<Response<InitializeResult, InitializeError>> {
    client
        .request::<Initialize, InitializeError>(InitializeParams::default(), 1)
        .await
}

pub fn handle_response<R: Serialize, E: Serialize>(response: Result<Response<R, E>>) {
    match response {
        Ok(response) => match response.result {
            JsonRPCResult::Result(result) => {
                println!("{}", serde_json::to_string_pretty(&result).unwrap())
            }
            JsonRPCResult::Error(err) => {
                println!("{}", err.message);

                if let Some(data) = err.data {
                    println!("{}", serde_json::to_string_pretty(&data).unwrap());
                }
            }
        },
        Err(err) => {
            println!("{}", err);
        }
    }
}
