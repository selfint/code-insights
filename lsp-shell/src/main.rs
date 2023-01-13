mod commands;
use anyhow::Result;

use commands::{InitializeCmd, NotificationCommand, RequestCommand};
use lsp_client::{
    jsonrpc_types::{JsonRPCResult, Response},
    lsp_types::request::Request as LspRequest,
    server_proxy::StdIOProxy,
    LspClient,
};
use serde::{de::DeserializeOwned, Serialize};
use std::io::{self, Write};

struct State {
    pub client: Option<LspClient>,
    pub exit: bool,
    pub request_id: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut state = State {
        client: None,
        exit: false,
        request_id: 0,
    };

    loop {
        print!("lsp-shell $ ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error reading from STDIN");

        state = handle_input(state, &input).await;
        if state.exit {
            return;
        }
    }
}

async fn handle_input(mut state: State, input: &str) -> State {
    match input
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => {}
        ["exit" | "quit" | "q"] => state.exit = true,
        ["help" | "h"] => handle_help(),
        ["start" | "s", cmd @ ..] => handle_start(&mut state, cmd),
        ["request" | "req" | "r", request @ ..] => handle_request(&mut state, request).await,
        ["notify" | "not" | "n", notification @ ..] => handle_notification(&state, notification),
        [cmd, ..] => println!("unknown command: '{}'", cmd),
    };

    state
}

fn handle_help() {
    println!("# TBD #")
}

fn handle_start(state: &mut State, cmd: &[&str]) {
    let program = cmd.first().unwrap();
    let args = cmd.iter().skip(1).collect::<Vec<_>>();

    let proc = tokio::process::Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start server");

    let stdin = proc.stdin.unwrap();
    let stdout = proc.stdout.unwrap();
    let stderr = proc.stderr.unwrap();

    let client = LspClient::new(&StdIOProxy::new(stdin, stdout, stderr));

    state.client = Some(client);
}

fn handle_notification(state: &State, notification: &[&str]) {
    println!("got notification: {:?}", notification);
}

async fn handle_request(state: &mut State, request: &[&str]) {
    let Some(ref client) = state.client else {
        println!("LSP client is not initialized, can't send request.");
        return;
    };

    println!("got request: {:?}", request);

    let request_type = request.first().unwrap();
    let request_args = request.iter().skip(1).copied().collect::<Vec<_>>();

    state.request_id += 1;
    let id = state.request_id;

    let response = match *request_type {
        "initialize" => handle_request_cmd::<InitializeCmd>(client, request_args, id).await,
        other => {
            println!("Unknown request type: '{}'", other);
            state.request_id -= 1;
            return;
        }
    };

    handle_response(response);
}

fn handle_response<R: Serialize, E: Serialize>(response: Result<Response<R, E>>) {
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

async fn handle_request_cmd<R>(
    client: &LspClient,
    args: Vec<&str>,
    id: u64,
) -> Result<
    Response<
        <<R as RequestCommand>::RequestType as LspRequest>::Result,
        <R as RequestCommand>::ErrorType,
    >,
>
where
    R: RequestCommand,
{
    let params = R::build_parameters(args);

    client
        .request::<R::RequestType, R::ErrorType>(params, id)
        .await
}

fn handle_notification_cmd<N>(client: &LspClient, args: Vec<&str>) -> Result<()>
where
    N: NotificationCommand,
{
    let params = N::build_parameters(args);

    client.notify::<N::NotificationType>(params)
}
