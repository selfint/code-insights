use std::future::Future;

use anyhow::{anyhow, Result};
use lsp_client::{
    jsonrpc_types::{JsonRPCResult, Response},
    lsp_types::{
        notification::Notification as LspNotification,
        request::{Initialize, Request as LspRequest},
        InitializeError, InitializeParams, InitializeResult,
    },
    LspClient,
};
use serde::{de::DeserializeOwned, Serialize};

pub trait RequestCommand
where
    Self::ErrorType: Serialize + DeserializeOwned,
    Self::RequestType: LspRequest,
{
    type RequestType;
    type ErrorType;

    fn build_parameters(args: Vec<&str>) -> <Self::RequestType as LspRequest>::Params;
}

pub trait NotificationCommand
where
    Self::NotificationType: LspNotification,
{
    type NotificationType;

    fn build_parameters(args: Vec<&str>) -> <Self::NotificationType as LspNotification>::Params;
}

pub struct InitializeCmd;

impl RequestCommand for InitializeCmd {
    type ErrorType = InitializeError;
    type RequestType = Initialize;

    fn build_parameters(args: Vec<&str>) -> <Self::RequestType as LspRequest>::Params {
        InitializeParams::default()
    }
}
