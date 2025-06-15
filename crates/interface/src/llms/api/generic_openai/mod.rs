use super::{
    client::ApiClient,
    config::{ApiConfig, ApiConfigTrait},
    openai::completion::OpenAICompletionRequest,
};
use crate::requests::{
    completion::{
        error::CompletionError, request::CompletionRequest, response::CompletionResponse,
    },
    embeddings::{EmbeddingsError, EmbeddingsRequest, EmbeddingsResponse},
};
use alith_devices::logging::LoggingConfig;
use alith_models::api_model::ApiLLMModel;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;

pub struct GenericApiBackend {
    pub(crate) client: ApiClient<GenericApiConfig>,
    pub model: ApiLLMModel,
}

impl GenericApiBackend {
    pub fn new(mut config: GenericApiConfig, model: ApiLLMModel) -> crate::Result<Self> {
        config.logging_config.load_logger()?;
        if let Ok(api_key) = config.api_config.load_api_key() {
            config.api_config.api_key = Some(api_key);
        }
        Ok(Self {
            client: ApiClient::new(config),
            model,
        })
    }

    pub(crate) async fn completion_request(
        &self,
        request: &CompletionRequest,
    ) -> crate::Result<CompletionResponse, CompletionError> {
        match self
            .client
            .post(
                &self.client.config.completion_path,
                OpenAICompletionRequest::new(request)?,
            )
            .await
        {
            Err(e) => Err(CompletionError::ClientError(e)),
            Ok(res) => Ok(CompletionResponse::new_from_openai(request, res)?),
        }
    }

    pub(crate) async fn embeddings_request(
        &self,
        request: &EmbeddingsRequest,
    ) -> crate::Result<EmbeddingsResponse, EmbeddingsError> {
        match self
            .client
            .post(
                "/embeddings",
                json!({
                    "input": request.input,
                    "model": request.model,
                }),
            )
            .await
        {
            Ok(res) => Ok(res),
            Err(e) => Err(EmbeddingsError::ClientError(e)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenericApiConfig {
    pub api_config: ApiConfig,
    pub logging_config: LoggingConfig,
    pub completion_path: String,
    pub extra_headers: HeaderMap,
}

impl Default for GenericApiConfig {
    fn default() -> Self {
        Self {
            api_config: ApiConfig {
                host: Default::default(),
                port: None,
                api_key: None,
                api_key_env_var: Default::default(),
            },
            logging_config: LoggingConfig {
                logger_name: "generic".to_string(),
                ..Default::default()
            },
            completion_path: "/chat/completions".to_string(),
            extra_headers: Default::default(),
        }
    }
}

impl GenericApiConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn completion_path<S: Into<String>>(mut self, path: S) -> Self {
        self.completion_path = path.into();
        self
    }
}

impl ApiConfigTrait for GenericApiConfig {
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(api_key) = self.api_key() {
            if let Ok(header_value) =
                HeaderValue::from_str(&format!("Bearer {}", api_key.expose_secret()))
            {
                headers.insert(AUTHORIZATION, header_value);
            } else {
                crate::error!("Failed to create header value from authorization value");
            }
        }

        for (k, v) in &self.extra_headers {
            headers.insert(k.clone(), v.clone());
        }

        headers
    }

    fn url(&self, path: &str) -> String {
        if let Some(port) = &self.api_config.port {
            format!("https://{}:{}{}", self.api_config.host, port, path)
        } else {
            format!("https://{}:{}", self.api_config.host, path)
        }
    }

    fn api_key(&self) -> &Option<SecretString> {
        &self.api_config.api_key
    }
}
