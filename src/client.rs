use async_trait::async_trait;

#[allow(unused_imports)]
use bytes::{BytesMut, BufMut};
#[cfg(feature = "hyper")]
use hyper::body::HttpBody;


#[async_trait(?Send)]
pub trait ComciganClient {
    async fn fetch_bytes(&self, url: String, target: &mut BytesMut) -> anyhow::Result<()>;
}

#[cfg(feature = "hyper")]
pub struct HyperClient {
    client: hyper::Client<hyper::client::HttpConnector>
}

#[async_trait(?Send)]
#[cfg(feature = "hyper")]
impl ComciganClient for HyperClient {
    async fn fetch_bytes(&self, url: String, target: &mut BytesMut) -> anyhow::Result<()> {
        let request = url.parse()?;
        let mut response = self.client.get(request).await?;

        while let Some(chunk) = response.body_mut().data().await {
            target.put(&chunk?[..]);
        }

        Ok(())
    }
}

#[cfg(feature = "hyper")]
impl HyperClient {
    pub fn new() -> HyperClient {
        HyperClient {
            client: hyper::Client::new()
        }
    }
}

#[cfg(feature = "wasm")]
pub struct WasmClient {
    pub proxy: String
}

#[async_trait(?Send)]
#[cfg(feature = "wasm")]
impl ComciganClient for WasmClient {
    async fn fetch_bytes(&self, url: String, target: &mut BytesMut) -> anyhow::Result<()> {
        let fetched_data = gloo_net::http::Request::get(format!("{}{}", self.proxy, url).as_str())
            .send()
            .await
            .unwrap()
            .binary()
            .await
            .unwrap();

        target.put(&fetched_data[..]);
        Ok(())
    }
}

#[cfg(feature = "wasm")]
impl WasmClient {
    pub fn new(proxy: String) -> WasmClient {
        WasmClient { proxy }
    }
}