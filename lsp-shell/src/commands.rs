use lsp_client::lsp_types::{
    notification::{Initialized, Notification as LspNotification},
    request::{Initialize, Request as LspRequest},
    InitializeError, InitializeParams, InitializedParams,
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

pub struct InitializedCmd;

impl NotificationCommand for InitializedCmd {
    type NotificationType = Initialized;

    fn build_parameters(args: Vec<&str>) -> <Self::NotificationType as LspNotification>::Params {
        InitializedParams {}
    }
}
