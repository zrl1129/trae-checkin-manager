use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct CdpClient {
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    next_id: u64,
}

impl CdpClient {
    pub async fn connect(port: u16) -> Result<Self> {
        let url = format!("http://127.0.0.1:{}/json", port);
        let resp = reqwest::get(&url).await?;
        let targets: Vec<Value> = resp.json().await?;

        let page_target = targets
            .iter()
            .find(|t| t.get("type").and_then(|v| v.as_str()) == Some("page"))
            .ok_or_else(|| anyhow::anyhow!("未找到 page 类型的 CDP target"))?;

        let ws_url = page_target["webSocketDebuggerUrl"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("webSocketDebuggerUrl 不存在"))?;

        let (ws_stream, _) = connect_async(ws_url).await?;

        let mut client = Self {
            ws_stream,
            next_id: 0,
        };

        client
            .send_command("Runtime.enable", serde_json::json!({}))
            .await?;

        Ok(client)
    }

    pub async fn send_command(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id + 1;
        self.next_id = id;

        let msg = serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        });

        let text = msg.to_string();
        self.ws_stream.send(Message::Text(text.into())).await?;

        loop {
            match self.ws_stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let resp: Value = serde_json::from_str(&text)?;
                    if resp.get("id").and_then(|v| v.as_u64()) == Some(id) {
                        if let Some(error) = resp.get("error") {
                            return Err(anyhow::anyhow!("CDP error: {}", error));
                        }
                        return Ok(resp);
                    }
                }
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => continue,
                Some(Ok(Message::Close(_))) => {
                    return Err(anyhow::anyhow!("CDP WebSocket 已关闭"));
                }
                Some(Ok(_)) => continue,
                Some(Err(e)) => {
                    return Err(anyhow::anyhow!("WebSocket 错误: {}", e));
                }
                None => return Err(anyhow::anyhow!("WebSocket 流结束")),
            }
        }
    }

    pub async fn evaluate(&mut self, expression: &str) -> Result<Value> {
        let resp = self
            .send_command(
                "Runtime.evaluate",
                serde_json::json!({
                    "expression": expression,
                    "returnByValue": true,
                }),
            )
            .await?;

        if let Some(exception) = resp.get("result").and_then(|r| r.get("exceptionDetails")) {
            let text = exception
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown error");
            return Err(anyhow::anyhow!("JS 执行错误: {}", text));
        }

        let result = &resp["result"]["result"];

        if let Some(value) = result.get("value") {
            Ok(value.clone())
        } else if result.get("type").and_then(|v| v.as_str()) == Some("undefined") {
            Ok(Value::Null)
        } else {
            Ok(result.clone())
        }
    }

    pub async fn close(&mut self) {
        let _ = self.ws_stream.close(None).await;
    }
}
