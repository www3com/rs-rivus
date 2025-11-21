use anyhow::Result;
use futures_util::StreamExt;
use reqwest::{Client, Method, header, ClientBuilder, Proxy};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Duration;

/// A robust HTTP client for production use.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    max_retries: u32,
    retry_delay: Duration,
    proxy_url: Option<String>,
}

impl HttpClient {
    /// Creates a new builder for `HttpClient`.
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    /// Returns the configured proxy URL, if any.
    pub fn proxy_url(&self) -> Option<&str> {
        self.proxy_url.as_deref()
    }

    /// Sends a generic HTTP request with retry logic.
    async fn send_request<T: Serialize + ?Sized>(
        &self,
        method: Method,
        url: &str,
        body: Option<&T>,
    ) -> Result<reqwest::Response> {
        for attempt in 1..=self.max_retries + 1 {
            let mut req = self.client.request(method.clone(), url);
            if let Some(b) = body {
                req = req.json(b);
            }

            let response = req.send().await;

            let should_retry = match response {
                Ok(resp) if resp.status().is_success() => {
                    return Ok(resp);
                }
                Ok(resp) if resp.status().is_server_error() && attempt <= self.max_retries => {
                    true
                }
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!("HTTP error: {} - {}", status, text));
                }
                Err(e) if e.is_timeout() && attempt <= self.max_retries => {
                    true
                }
                Err(e) => return Err(anyhow::anyhow!("Request failed: {}", e)),
            };

            if should_retry {
                tokio::time::sleep(self.retry_delay).await;
            }
        }

        Err(anyhow::anyhow!("Max retries ({}) reached", self.max_retries))
    }

    /// Sends a GET request and returns the response as JSON.
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.send_request::<()>(Method::GET, url, None).await?;
        Ok(response.json::<T>().await?)
    }

    /// Sends a GET request and returns the response as string.
    pub async fn get_string(&self, url: &str) -> Result<String> {
        let response = self.send_request::<()>(Method::GET, url, None).await?;
        Ok(response.text().await?)
    }

    /// Sends a POST request and returns the response as JSON.
    pub async fn post<T: Serialize, R: DeserializeOwned>(&self, url: &str, body: &T) -> Result<R> {
        let response = self.send_request(Method::POST, url, Some(body)).await?;
        Ok(response.json::<R>().await?)
    }

    /// Sends a POST request and returns the response as string.
    pub async fn post_string<T: Serialize>(&self, url: &str, body: &T) -> Result<String> {
        let response = self.send_request(Method::POST, url, Some(body)).await?;
        Ok(response.text().await?)
    }

    /// Sends a PUT request and returns the response as JSON.
    pub async fn put<T: Serialize, R: DeserializeOwned>(&self, url: &str, body: &T) -> Result<R> {
        let response = self.send_request(Method::PUT, url, Some(body)).await?;
        Ok(response.json::<R>().await?)
    }

    /// Sends a PUT request and returns the response as string.
    pub async fn put_string<T: Serialize>(&self, url: &str, body: &T) -> Result<String> {
        let response = self.send_request(Method::PUT, url, Some(body)).await?;
        Ok(response.text().await?)
    }

    /// Sends a DELETE request and returns the response as JSON.
    pub async fn delete<R: DeserializeOwned>(&self, url: &str) -> Result<R> {
        let response = self.send_request::<()>(Method::DELETE, url, None).await?;
        Ok(response.json::<R>().await?)
    }

    /// Sends a DELETE request and returns the response as string.
    pub async fn delete_string(&self, url: &str) -> Result<String> {
        let response = self.send_request::<()>(Method::DELETE, url, None).await?;
        Ok(response.text().await?)
    }

    /// Downloads a file using streaming and saves it to the specified path.
    pub async fn download(&self, url: &str, out_dir: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Download failed {}: HTTP status {}", url, response.status()));
        }

        // 从响应头中获取文件名
        let filename = response
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .and_then(|value| {
                value
                    .to_str()
                    .ok()
                    .and_then(|s| {
                        s.split("filename=")
                            .nth(1)
                            .map(|s| s.trim_matches(|c| c == '"' || c == '\''))
                    })
            })
            .unwrap_or_else(|| {
                // 如果响应头中没有文件名，则从 URL 中提取
                url.split('/')
                    .last()
                    .unwrap_or("downloaded_file")
            });

        // 构建完整的文件路径
        let full_path = Path::new(out_dir).join(filename);
        let path = full_path.as_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(path)?;
        let mut file = BufWriter::with_capacity(1024 * 1024, file); // 1MB buffer
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
        }

        file.flush()?;
        Ok(path.canonicalize()?.display().to_string())
    }
}

/// Builder for configuring `HttpClient`.
#[derive(Debug)]
pub struct HttpClientBuilder {
    headers: header::HeaderMap,
    connect_timeout: Duration,
    timeout: Duration,
    max_retries: u32,
    retry_delay: Duration,
    pool_max_idle_per_host: usize,
    proxy_url: Option<String>,
}

impl HttpClientBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

        Self {
            headers,
            connect_timeout: Duration::from_secs(5),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            pool_max_idle_per_host: 50,
            proxy_url: None,
        }
    }


    /// Appends a header value, allowing multiple values for the same header name.
    /// This is useful for headers like 'Set-Cookie' that can have multiple values.
    pub fn append_header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: TryInto<header::HeaderName>,
        V: TryInto<header::HeaderValue>,
        K::Error: std::fmt::Debug,
        V::Error: std::fmt::Debug,
    {
        let header_name = key.try_into()
            .map_err(|e| anyhow::anyhow!("Invalid header key: {:?}", e))?;
        let header_value = value.try_into()
            .map_err(|e| anyhow::anyhow!("Invalid header value: {:?}", e))?;
        
        // 验证头部值是否有效
        if !header_value.as_bytes().iter().all(|&b| b >= 32 && b != 127) {
            return Err(anyhow::anyhow!("Header value contains invalid characters"));
        }

        self.headers.append(header_name, header_value);
        Ok(self)
    }

    /// Sets the connection timeout.
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = duration;
        self
    }

    /// Sets the overall request timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// Sets the maximum number of retries.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Sets the delay between retries.
    pub fn retry_delay(mut self, duration: Duration) -> Self {
        self.retry_delay = duration;
        self
    }

    /// Sets the maximum number of idle connections per host.
    pub fn pool_max_idle_per_host(mut self, max: usize) -> Self {
        self.pool_max_idle_per_host = max;
        self
    }

    /// Sets a proxy URL (e.g., "http://proxy.example.com:8080").
    /// If `None`, no proxy is used.
    pub fn proxy_url(mut self, url: Option<impl Into<String>>) -> Self {
        self.proxy_url = url.map(Into::into);
        self
    }

    /// Builds the `HttpClient`.
    pub fn build(self) -> Result<HttpClient> {
        let mut builder = ClientBuilder::new()
            .default_headers(self.headers)
            .connect_timeout(self.connect_timeout)
            .timeout(self.timeout)
            .pool_max_idle_per_host(self.pool_max_idle_per_host);

        if let Some(proxy_url) = &self.proxy_url {
            let proxy = Proxy::all(proxy_url)
                .map_err(|e| anyhow::anyhow!("Invalid proxy URL '{}': {}", proxy_url, e))?;
            builder = builder.proxy(proxy);
        }

        let client = builder.build()?;
        Ok(HttpClient {
            client,
            max_retries: self.max_retries,
            retry_delay: self.retry_delay,
            proxy_url: self.proxy_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Post {
        user_id: i32,
        id: i32,
        title: String,
        body: String,
    }

    #[tokio::test]
    async fn test_get_json() -> Result<()> {
        let client = HttpClient::builder().build()?;
        let post: Post = client.get("https://jsonplaceholder.typicode.com/posts/1").await?;
        assert_eq!(post.id, 1);
        assert_eq!(post.user_id, 1);
        assert_eq!(post.title, "sunt aut facere repellat provident occaecati excepturi optio reprehenderit");
        assert!(post.body.contains("quia et suscipit"));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_string() -> Result<()> {
        let client = HttpClient::builder().build()?;
        let text = client.get_string("https://jsonplaceholder.typicode.com/posts/1").await?;
        assert!(!text.is_empty());
        assert!(text.contains("userId"));
        Ok(())
    }
}