//! agit-ai — AI-powered commit message generation for agit.
//!
//! 支持 OpenAI-compatible API，用于自动生成 commit message。
//!
//! ## 配置方式（按优先级）
//!
//! 1. 环境变量 `AGIT_LLM_API_KEY` / `AGIT_LLM_PROVIDER` / `AGIT_LLM_MODEL`
//! 2. 仓库级 `.agit/config.toml` 的 `[llm]` 段
//! 3. 全局 `~/.agitconfig.toml` 的 `[llm]` 段
//! 4. 默认：provider=openai, model=gpt-4o-mini
//!
//! ## 配置文件示例
//!
//! ```toml
//! [llm]
//! api_key = "sk-xxx"
//! provider = "deepseek"   # openai / deepseek / anthropic / moonshot / zhipu / ollama
//! model = "deepseek-chat" # 可选，不填自动匹配 provider 默认模型
//! ```
//!
//! `AGIT_LLM_API_URL` 环境变量可直接覆盖 API 端点（优先级最高）。

use agit_core::config;
use serde::{Deserialize, Serialize};

/// LLM API 运行时配置（已解析）。
#[derive(Clone)]
pub struct LlmConfig {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
}

impl LlmConfig {
    /// 从 Config 系统加载 LLM 配置。
    /// 优先级：AGIT_LLM_API_URL 环境变量 > provider 预设 > 默认 OpenAI
    pub fn from_config(cfg: &config::Config) -> Option<Self> {
        let api_key = cfg.llm.api_key.clone()?;

        // 解析 API URL
        let api_url = std::env::var("AGIT_LLM_API_URL").ok().unwrap_or_else(|| {
            // 根据 provider 查预设
            if let Some(ref provider) = cfg.llm.provider {
                if let Some((url, _)) = config::resolve_llm_provider(provider) {
                    return url.to_string();
                }
            }
            // 默认 OpenAI
            "https://api.openai.com/v1".to_string()
        });

        let model = cfg.llm.model.clone().unwrap_or_else(|| {
            // 根据 provider 查默认 model
            if let Some(ref provider) = cfg.llm.provider {
                if let Some((_, default_model)) = config::resolve_llm_provider(provider) {
                    return default_model.to_string();
                }
            }
            "gpt-4o-mini".to_string()
        });

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
pub fn chat_completion(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

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
        return Err(format!(
            "LLM API error ({}): {}\n\
             Hint: check AGIT_LLM_API_KEY and AGIT_LLM_PROVIDER.\n\
             Supported providers: openai, deepseek, anthropic, moonshot, zhipu, ollama",
            status, body
        )
        .into());
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
