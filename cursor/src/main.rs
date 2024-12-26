use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use chrono::{DateTime, Local, Utc};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// 应用状态
struct AppState {
    start_time: DateTime<Local>,
    version: String,
    total_requests: u64,
    active_requests: u64,
    request_logs: Vec<RequestLog>,
    route_prefix: String,
    token_infos: Vec<TokenInfo>,
}

// 模型定义
#[derive(Serialize, Deserialize, Clone)]
struct Model {
    id: String,
    created: i64,
    object: String,
    owned_by: String,
}

// 请求日志
#[derive(Serialize, Clone)]
struct RequestLog {
    timestamp: DateTime<Local>,
    model: String,
    checksum: String,
    auth_token: String,
    stream: bool,
}

// 聊天请求
#[derive(Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(default)]
    stream: bool,
}

// 添加用于请求的消息结构体
#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

// 支持的模型列表
mod models;
use models::AVAILABLE_MODELS;

// 用于存储 token 信息
#[derive(Debug)]
struct TokenInfo {
    token: String,
    checksum: String,
}

// TokenUpdateRequest 结构体
#[derive(Deserialize)]
struct TokenUpdateRequest {
    tokens: String,
    #[serde(default)]
    token_list: Option<String>,
}

// 自定义错误类型
#[derive(Debug)]
enum ChatError {
    ModelNotSupported(String),
    EmptyMessages,
    StreamNotSupported(String),
    NoTokens,
    RequestFailed(String),
    Unauthorized,
}

impl ChatError {
    fn to_json(&self) -> serde_json::Value {
        let (code, message) = match self {
            ChatError::ModelNotSupported(model) => (
                "model_not_supported",
                format!("Model '{}' is not supported", model),
            ),
            ChatError::EmptyMessages => (
                "empty_messages",
                "Message array cannot be empty".to_string(),
            ),
            ChatError::StreamNotSupported(model) => (
                "stream_not_supported",
                format!("Streaming is not supported for model '{}'", model),
            ),
            ChatError::NoTokens => ("no_tokens", "No available tokens".to_string()),
            ChatError::RequestFailed(err) => ("request_failed", format!("Request failed: {}", err)),
            ChatError::Unauthorized => ("unauthorized", "Invalid authorization token".to_string()),
        };

        serde_json::json!({
            "error": {
                "code": code,
                "message": message
            }
        })
    }
}

#[tokio::main]
async fn main() {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 处理 token 文件路径
    let token_file = std::env::var("TOKEN_FILE").unwrap_or_else(|_| ".token".to_string());

    // 加载 tokens
    let token_infos = load_tokens(&token_file);

    // 获取路由前缀配置
    let route_prefix = std::env::var("ROUTE_PREFIX").unwrap_or_default();

    // 初始化应用状态
    let state = Arc::new(Mutex::new(AppState {
        start_time: Local::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        total_requests: 0,
        active_requests: 0,
        request_logs: Vec::new(),
        route_prefix: route_prefix.clone(),
        token_infos,
    }));

    // 设置路由
    let app = Router::new()
        .route("/", get(handle_root))
        .route("/tokeninfo", get(handle_tokeninfo_page))
        .route(&format!("{}/v1/models", route_prefix), get(handle_models))
        .route("/checksum", get(handle_checksum))
        .route("/update-tokeninfo", get(handle_update_tokeninfo))
        .route("/get-tokeninfo", post(handle_get_tokeninfo))
        .route("/update-tokeninfo", post(handle_update_tokeninfo_post))
        .route(
            &format!("{}/v1/chat/completions", route_prefix),
            post(handle_chat),
        )
        .route("/logs", get(handle_logs))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 启动服务器
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("服务器运行在端口 {}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Token 加载函数
fn load_tokens(token_file: &str) -> Vec<TokenInfo> {
    let token_list_file =
        std::env::var("TOKEN_LIST_FILE").unwrap_or_else(|_| ".token-list".to_string());

    // 读取并规范化 .token 文件
    let tokens = if let Ok(content) = std::fs::read_to_string(token_file) {
        let normalized = content.replace("\r\n", "\n");
        if normalized != content {
            std::fs::write(token_file, &normalized).unwrap();
        }
        normalized
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                let parts: Vec<&str> = line.split("::").collect();
                match parts.len() {
                    1 => Some(line.to_string()),
                    2 => Some(parts[1].to_string()),
                    _ => {
                        println!("警告: 第{}行包含多个'::'分隔符,已忽略此行", idx + 1);
                        None
                    }
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    } else {
        eprintln!("警告: 无法读取token文件 '{}'", token_file);
        Vec::new()
    };

    // 读取现有的 token-list
    let mut token_map: std::collections::HashMap<String, String> =
        if let Ok(content) = std::fs::read_to_string(&token_list_file) {
            content
                .split('\n')
                .filter(|s| !s.is_empty())
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            std::collections::HashMap::new()
        };

    // 为新 token 生成 checksum
    for token in tokens {
        if !token_map.contains_key(&token) {
            let checksum = cursor_api::generate_checksum(
                &cursor_api::generate_hash(),
                Some(&cursor_api::generate_hash()),
            );
            token_map.insert(token, checksum);
        }
    }

    // 更新 token-list 文件
    let token_list_content = token_map
        .iter()
        .map(|(token, checksum)| format!("{},{}", token, checksum))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(token_list_file, token_list_content).unwrap();

    // 转换为 TokenInfo vector
    token_map
        .into_iter()
        .map(|(token, checksum)| TokenInfo { token, checksum })
        .collect()
}

// 根路由处理
async fn handle_root(State(state): State<Arc<Mutex<AppState>>>) -> Json<serde_json::Value> {
    let state = state.lock().await;
    let uptime = (Local::now() - state.start_time).num_seconds();

    Json(serde_json::json!({
        "status": "healthy",
        "version": state.version,
        "uptime": uptime,
        "stats": {
            "started": state.start_time,
            "totalRequests": state.total_requests,
            "activeRequests": state.active_requests,
            "memory": {
                "heapTotal": 0,
                "heapUsed": 0,
                "rss": 0
            }
        },
        "models": AVAILABLE_MODELS.iter().map(|m| &m.id).collect::<Vec<_>>(),
        "endpoints": [
            &format!("{}/v1/chat/completions", state.route_prefix),
            &format!("{}/v1/models", state.route_prefix),
            "/checksum",
            "/tokeninfo",
            "/update-tokeninfo",
            "/get-tokeninfo"
        ]
    }))
}

async fn handle_tokeninfo_page() -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "text/html")
        .body(include_str!("../static/tokeninfo.min.html").to_string())
        .unwrap()
}

// 模型列表处理
async fn handle_models() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "object": "list",
        "data": AVAILABLE_MODELS.to_vec()
    }))
}

// Checksum 处理
async fn handle_checksum() -> Json<serde_json::Value> {
    let checksum = cursor_api::generate_checksum(
        &cursor_api::generate_hash(),
        Some(&cursor_api::generate_hash()),
    );
    Json(serde_json::json!({
        "checksum": checksum
    }))
}

// 更新 TokenInfo 处理
async fn handle_update_tokeninfo(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<serde_json::Value> {
    // 获取当前的 token 文件路径
    let token_file = std::env::var("TOKEN_FILE").unwrap_or_else(|_| ".token".to_string());

    // 重新加载 tokens
    let token_infos = load_tokens(&token_file);

    // 更新应用状态
    {
        let mut state = state.lock().await;
        state.token_infos = token_infos;
    }

    Json(serde_json::json!({
        "status": "success",
        "message": "Token list has been reloaded"
    }))
}

// 获取 TokenInfo 处理
async fn handle_get_tokeninfo(
    State(_state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let env_token = std::env::var("AUTH_TOKEN").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if auth_header != env_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 获取文件路径
    let token_file = std::env::var("TOKEN_FILE").unwrap_or_else(|_| ".token".to_string());
    let token_list_file =
        std::env::var("TOKEN_LIST_FILE").unwrap_or_else(|_| ".token-list".to_string());

    // 读取文件内容
    let tokens = std::fs::read_to_string(&token_file).unwrap_or_else(|_| String::new());
    let token_list = std::fs::read_to_string(&token_list_file).unwrap_or_else(|_| String::new());

    Ok(Json(serde_json::json!({
        "status": "success",
        "token_file": token_file,
        "token_list_file": token_list_file,
        "tokens": tokens,
        "token_list": token_list
    })))
}

async fn handle_update_tokeninfo_post(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<TokenUpdateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // 验证 AUTH_TOKEN
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let env_token = std::env::var("AUTH_TOKEN").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if auth_header != env_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 获取文件路径
    let token_file = std::env::var("TOKEN_FILE").unwrap_or_else(|_| ".token".to_string());
    let token_list_file =
        std::env::var("TOKEN_LIST_FILE").unwrap_or_else(|_| ".token-list".to_string());

    // 写入 .token 文件
    std::fs::write(&token_file, &request.tokens).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 如果提供了 token_list，则写入
    if let Some(token_list) = request.token_list {
        std::fs::write(&token_list_file, token_list)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // 重新加载 tokens
    let token_infos = load_tokens(&token_file);
    let token_infos_len = token_infos.len();

    // 更新应用状态
    {
        let mut state = state.lock().await;
        state.token_infos = token_infos;
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Token files have been updated and reloaded",
        "token_file": token_file,
        "token_list_file": token_list_file,
        "token_count": token_infos_len
    })))
}

// 日志处理
async fn handle_logs(State(state): State<Arc<Mutex<AppState>>>) -> Json<serde_json::Value> {
    let state = state.lock().await;
    Json(serde_json::json!({
        "total": state.request_logs.len(),
        "logs": state.request_logs,
        "timestamp": Utc::now(),
        "status": "success"
    }))
}

// 聊天处理函数的签名
async fn handle_chat(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(request): Json<ChatRequest>,
) -> Result<Response<Body>, (StatusCode, Json<serde_json::Value>)> {
    // 验证模型是否支持
    if !AVAILABLE_MODELS.iter().any(|m| m.id == request.model) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ChatError::ModelNotSupported(request.model.clone()).to_json()),
        ));
    }

    let request_time = Local::now();

    // 验证请求
    if request.messages.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ChatError::EmptyMessages.to_json()),
        ));
    }

    // 验证 O1 模型不支持流式输出
    if request.model.starts_with("o1") && request.stream {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ChatError::StreamNotSupported(request.model.clone()).to_json()),
        ));
    }

    // 获取并处理认证令牌
    let auth_token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json(ChatError::Unauthorized.to_json()),
        ))?;

    // 验证环境变量中的 AUTH_TOKEN
    if let Ok(env_token) = std::env::var("AUTH_TOKEN") {
        if auth_token != env_token {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ChatError::Unauthorized.to_json()),
            ));
        }
    }

    // 完整的令牌处理逻辑和对应的 checksum
    let (auth_token, checksum) = {
        static CURRENT_KEY_INDEX: AtomicUsize = AtomicUsize::new(0);
        let state_guard = state.lock().await;
        let token_infos = &state_guard.token_infos;

        if token_infos.is_empty() {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ChatError::NoTokens.to_json()),
            ));
        }

        let index = CURRENT_KEY_INDEX.fetch_add(1, Ordering::SeqCst) % token_infos.len();
        let token_info = &token_infos[index];
        (token_info.token.clone(), token_info.checksum.clone())
    };

    // 更新请求日志
    {
        let mut state = state.lock().await;
        state.total_requests += 1;
        state.active_requests += 1;
        state.request_logs.push(RequestLog {
            timestamp: request_time,
            model: request.model.clone(),
            checksum: checksum.clone(),
            auth_token: auth_token.clone(),
            stream: request.stream,
        });

        if state.request_logs.len() > 100 {
            state.request_logs.remove(0);
        }
    }

    // 消息转换
    let chat_inputs: Vec<cursor_api::ChatInput> = request
        .messages
        .into_iter()
        .map(|m| cursor_api::ChatInput {
            role: m.role,
            content: m.content,
        })
        .collect();

    // 将消息转换为hex格式
    let hex_data = cursor_api::encode_chat_message(chat_inputs, &request.model)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    ChatError::RequestFailed("Failed to encode chat message".to_string()).to_json(),
                ),
            )
        })?;

    // 构建请求客户端
    let client = Client::new();
    let request_id = Uuid::new_v4().to_string();
    let response = client
        .post("https://api2.cursor.sh/aiserver.v1.AiService/StreamChat")
        .header("Content-Type", "application/connect+proto")
        .header("Authorization", format!("Bearer {}", auth_token))
        .header("connect-accept-encoding", "gzip,br")
        .header("connect-protocol-version", "1")
        .header("user-agent", "connect-es/1.4.0")
        .header("x-amzn-trace-id", format!("Root={}", &request_id))
        .header("x-cursor-checksum", &checksum)
        .header("x-cursor-client-version", "0.42.5")
        .header("x-cursor-timezone", "Asia/Shanghai")
        .header("x-ghost-mode", "false")
        .header("x-request-id", &request_id)
        .header("Host", "api2.cursor.sh")
        .body(hex_data)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ChatError::RequestFailed(format!("Request failed: {}", e)).to_json()),
            )
        })?;

    // 释放活动请求计数
    {
        let mut state = state.lock().await;
        state.active_requests -= 1;
    }

    if request.stream {
        let response_id = format!("chatcmpl-{}", Uuid::new_v4());

        let stream = response.bytes_stream().then(move |chunk| {
            let response_id = response_id.clone();
            let model = request.model.clone();

            async move {
                let chunk = chunk.unwrap_or_default();
                let text = cursor_api::decode_response(&chunk).await;

                if text.is_empty() {
                    return Ok::<_, Infallible>(Bytes::from("[DONE]"));
                }

                let data = serde_json::json!({
                    "id": &response_id,
                    "object": "chat.completion.chunk",
                    "created": chrono::Utc::now().timestamp(),
                    "model": model,
                    "choices": [{
                        "index": 0,
                        "delta": {
                            "content": text
                        }
                    }]
                });

                Ok::<_, Infallible>(Bytes::from(format!("data: {}\n\n", data.to_string())))
            }
        });

        Ok(Response::builder()
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .body(Body::from_stream(stream))
            .unwrap())
    } else {
        // 非流式响应
        let mut full_text = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        ChatError::RequestFailed(format!("Failed to read response chunk: {}", e))
                            .to_json(),
                    ),
                )
            })?;
            full_text.push_str(&cursor_api::decode_response(&chunk).await);
        }

        // 处理文本
        full_text = full_text
            .replace(
                regex::Regex::new(r"^.*<\|END_USER\|>").unwrap().as_str(),
                "",
            )
            .replace(regex::Regex::new(r"^\n[a-zA-Z]?").unwrap().as_str(), "")
            .trim()
            .to_string();

        let response_data = serde_json::json!({
            "id": format!("chatcmpl-{}", Uuid::new_v4()),
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": request.model,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": full_text
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }
        });

        Ok(Response::new(Body::from(response_data.to_string())))
    }
}
