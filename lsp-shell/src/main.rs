mod request_handlers;

use lsp_client::{server_proxy::StdIOProxy, LspClient};
use std::io::{self, Write};

struct State {
    pub client: Option<LspClient>,
    pub exit: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut state = State {
        client: None,
        exit: false,
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
        ["request" | "req" | "r", request @ ..] => handle_request(&state, request).await,
        ["notify" | "not" | "n", notification @ ..] => handle_notification(&state, notification),
        [cmd, ..] => println!("unknown command: '{}'", cmd),
    };

    state
}

fn handle_help() {
    println!("# TBD #")
}

fn handle_start(state: &mut State, cmd: &[&str]) {
    let server_cmd = cmd.first().unwrap();
    let args = cmd.iter().skip(1).collect::<Vec<_>>();

    let proc = tokio::process::Command::new(server_cmd)
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

async fn handle_request(state: &State, request: &[&str]) {
    if let Some(ref client) = state.client {
        request_handlers::handle_request(client, request).await
    } else {
        println!("LSP client is not initialized, can't send request.");
    };
}

fn handle_notification(state: &State, notification: &[&str]) {
    println!("got notification: {:?}", notification);
}
