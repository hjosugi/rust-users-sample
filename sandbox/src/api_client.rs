//! A small JSON API client. It knows only the base URL.
//! The caller picks the path and the types, so it works for any payload.

use reqwest::RequestBuilder;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct ApiClient {
    // One `Client` for every call, so connections are reused.
    http: reqwest::Client,
    base: String,
}

impl ApiClient {
    /// `base` is like `http://127.0.0.1:3000`. A trailing `/` is fine.
    pub fn new(base: impl Into<String>) -> Self {
        let base: String = base.into();
        Self {
            http: reqwest::Client::new(),
            base: base.trim_end_matches('/').to_string(),
        }
    }

    /// `path` is like `/users` or `users`. Both work.
    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base,
            path.trim_start_matches('/')
        )
    }

    /// GET and parse the body as `T`.
    /// `T = serde_json::Value` takes any JSON.
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, reqwest::Error> {
        self.send(self.http.get(self.url(path))).await
    }

    /// POST `body` as JSON, parse the answer as `T`.
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, reqwest::Error> {
        self.send(self.http.post(self.url(path)).json(body))
            .await
    }

    /// PUT `body` as JSON, parse the answer as `T`.
    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, reqwest::Error> {
        self.send(self.http.put(self.url(path)).json(body))
            .await
    }

    /// DELETE. No body is read, because 204 has none.
    pub async fn delete(
        &self,
        path: &str,
    ) -> Result<(), reqwest::Error> {
        self.http
            .delete(self.url(path))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // Every method ends the same way: send, 4xx/5xx -> Err, JSON -> T.
    async fn send<T: DeserializeOwned>(
        &self,
        req: RequestBuilder,
    ) -> Result<T, reqwest::Error> {
        req.send().await?.error_for_status()?.json().await
    }
}
