use async_trait::async_trait;

#[allow(unused_imports)]
use bytes::{BytesMut, BufMut};
#[cfg(any(feature = "hyper"))]
use hyper::body::HttpBody;


#[async_trait(?Send)]
pub trait ComciganClient {
    async fn fetch_bytes(&self, url: String, target: &mut BytesMut) -> anyhow::Result<()>;
    async fn fetch_string(&self, url: String, target: &mut String) -> anyhow::Result<()>;
}

#[cfg(any(feature = "hyper"))]
pub struct HyperClient {
    client: hyper::Client<hyper::client::HttpConnector>
}

#[async_trait(?Send)]
#[cfg(any(feature = "hyper"))]
impl ComciganClient for HyperClient {
    async fn fetch_bytes(&self, url: String, target: &mut BytesMut) -> anyhow::Result<()> {
        let request = url.parse()?;
        let mut response = self.client.get(request).await?;

        while let Some(chunk) = response.body_mut().data().await {
            target.put(&chunk?[..]);
        }

        Ok(())
    }

    async fn fetch_string(&self, url: String, target: &mut String) -> anyhow::Result<()> {
        let mut buf = BytesMut::with_capacity(1024);
        let bytes = self.fetch_bytes(url, &mut buf).await?;
        let (string, _, _) = encoding_rs::UTF_8.decode(&buf[..]);

        target.push_str(&string);

        Ok(())
    }
}

#[cfg(any(feature = "hyper"))]
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

    
    async fn fetch_string(&self, url: String, target: &mut String) -> anyhow::Result<()> {
        let fetched_data = gloo_net::http::Request::get(format!("{}{}", self.proxy, url).as_str())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        log::info!("Raw: {}", fetched_data);

        target.push_str(fetched_data.as_str());

        Ok(())
    }
}

#[cfg(feature = "wasm")]
impl WasmClient {
    pub fn new(proxy: String) -> WasmClient {
        WasmClient { proxy }
    }
}