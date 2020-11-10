use anyhow::{Context, Result};
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Client {
    inner: reqwest::Client,
    url: Url,
}

impl Client {
    pub fn new(base_url: Url) -> Self {
        Self {
            inner: reqwest::Client::new(),
            url: base_url,
        }
    }

    pub async fn send<Req, Res>(&self, request: Request<Req>) -> Result<Res>
    where
        Req: Debug + Serialize,
        Res: Debug + DeserializeOwned,
    {
        self.send_with_path("".into(), request).await
    }

    pub async fn send_with_path<Req, Res>(&self, path: String, request: Request<Req>) -> Result<Res>
    where
        Req: Debug + Serialize,
        Res: Debug + DeserializeOwned,
    {
        let url = self.url.clone().join(&path)?;

        let response = self
            .inner
            .post(url.clone())
            .json(&request)
            .send()
            .await
            .with_context(|| format!("failed to send POST request to {}", self.url))?
            .json::<Response<Res>>()
            .await
            .context("failed to deserialize JSON response as JSON-RPC response")?
            .payload
            .into_result()
            .with_context(|| {
                format!(
                    "JSON-RPC request {} failed",
                    serde_json::to_string(&request).expect("can always serialize to JSON")
                )
            })?;

        Ok(response)
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct Request<T> {
    id: String,
    jsonrpc: String,
    method: String,
    params: T,
}

impl<T> Request<T> {
    pub fn new(method: &str, params: T, jsonrpc: String) -> Self {
        Self {
            id: "1".to_owned(),
            jsonrpc,
            method: method.to_owned(),
            params,
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct Response<R> {
    #[serde(flatten)]
    pub payload: ResponsePayload<R>,
}

#[derive(serde::Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponsePayload<R> {
    Result(R),
    Error(JsonRpcError),
}

impl<R> ResponsePayload<R> {
    fn into_result(self) -> Result<R, JsonRpcError> {
        match self {
            ResponsePayload::Result(result) => Ok(result),
            ResponsePayload::Error(e) => Err(e),
        }
    }
}

#[derive(Debug, serde::Deserialize, PartialEq, thiserror::Error)]
#[error("JSON-RPC request failed with code {code}: {message}")]
pub struct JsonRpcError {
    code: i64,
    message: String,
}

pub fn serialize<T>(t: T) -> Result<serde_json::Value>
where
    T: Serialize,
{
    let value = serde_json::to_value(t)?;

    Ok(value)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("URL Parse: ")]
    UrlParse(#[from] url::ParseError),
    #[error("failed to deserialize JSON response as JSON-RPC response: ")]
    Serde(#[from] serde_json::Error),
    #[error("connection:")]
    ConnectionFailed(#[from] reqwest::Error),
    #[error("JSON RPC: ")]
    JsonRpc(#[from] JsonRpcError),
}
