//! LLM API 调用模块（需 `ai` feature）。
//!
//! 支持 OpenAI-compatible API，用于自动生成 commit message。
//!
//! 环境变量：
//! - `AGIT_LLM_API_KEY` — API 密钥（必需）
//! - `AGIT_LLM_API_URL` — API 端点（默认 `https://api.openai.com/v1`）
//! - `AGIT_LLM_MODEL` — 模型名（默认 `gpt-4o-mini`）

use serde::{Deserialize, Serialize};

/// LLM API 配置。
#[derive(Clone)]
pub struct LlmConfig {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
}

impl LlmConfig {
    /// 从环境变量加载配置。
    /// 如果 `AGIT_LLM_API_KEY` 未设置则返回 None。
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("AGIT_LLM_API_KEY").ok()?;
        let api_url = std::env::var("AGIT_LLM_API_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("AGIT_LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Some(LlmConfig {
            api_key,
            api_url,
            model,
        })
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

/// 调用 LLM API 获取 chat completion。
///
/// 返回生成的文本内容，失败时返回错误。
pub fn chat_completion(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let request = ChatRequest {
        model: config.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: 0.3,
        max_tokens: 150,
    };

    let response = client
        .post(format!("{}/chat/completions", config.api_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("LLM API error ({}): {}", status, body).into());
    }

    let chat_response: ChatResponse = response.json()?;
    let content = chat_response
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_default();

    Ok(content)
}

/// 从 git diff 内容生成 commit message。
pub fn generate_commit_message(
    config: &LlmConfig,
    diff: &str,
    hint: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if diff.trim().is_empty() {
        return Ok("empty commit".to_string());
    }

    let system_prompt = "You are a git commit message generator. \
        Generate a concise, conventional commit message (feat:/fix:/docs:/refactor:/chore:/test:/style:) \
        based on the diff. Use present tense, keep under 72 chars for the first line. \
        Output ONLY the commit message, no explanation.";

    let user_prompt = if let Some(h) = hint {
        format!("User hint: {}\n\nDiff:\n{}", h, diff)
    } else {
        format!("Diff:\n{}", diff)
    };

    chat_completion(config, system_prompt, &user_prompt)
}
