use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use super::error::AiError;

/// 一次 HTTP 调用。adapter 负责把业务参数翻译成它。
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// 传输层抽象。好处是业务逻辑可以脱离真实网络做单元测试。
pub trait Transport: Send + Sync {
    fn send<'a>(&'a self, request: HttpRequest) -> BoxFuture<'a, Result<HttpResponse, AiError>>;
}

/// 真实实现：reqwest。
pub struct HttpTransport {
    client: reqwest::Client,
}

impl HttpTransport {
    pub fn new(timeout_secs: u64) -> Result<Self, AiError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|error| AiError::internal(format!("初始化网络客户端失败：{error}")))?;

        Ok(Self { client })
    }
}

impl Transport for HttpTransport {
    fn send<'a>(&'a self, request: HttpRequest) -> BoxFuture<'a, Result<HttpResponse, AiError>> {
        Box::pin(async move {
            let mut builder = self.client.post(&request.url);
            for (name, value) in &request.headers {
                builder = builder.header(name.as_str(), value.as_str());
            }

            let response = builder
                .body(request.body)
                .send()
                .await
                .map_err(|error| {
                    if error.is_timeout() {
                        AiError::network("请求超时")
                    } else if error.is_connect() {
                        AiError::network("连不上模型服务，可能是网络或代理问题")
                    } else {
                        AiError::network(error.to_string())
                    }
                })?;

            let status = response.status().as_u16();
            let body = response
                .text()
                .await
                .map_err(|error| AiError::network(error.to_string()))?;

            Ok(HttpResponse { status, body })
        })
    }
}
