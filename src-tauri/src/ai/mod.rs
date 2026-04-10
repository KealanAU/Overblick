use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<AiMessage>,
    pub max_tokens: u32,
}

#[derive(Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
pub struct OpenAiChoice {
    pub message: OpenAiChoiceMessage,
}

#[derive(Deserialize)]
pub struct OpenAiChoiceMessage {
    pub content: String,
}

#[derive(Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<AiMessage>,
}

#[derive(Deserialize)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryRow {
    pub repo: String,
    pub commits: i64,
    pub time_minutes: i64,
    pub highlight: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryResult {
    pub rows: Vec<SummaryRow>,
    pub overall: String,
    pub period_days: i64,
}

pub fn strip_json_fences(s: &str) -> &str {
    let s = s.trim();
    if let Some(inner) = s.strip_prefix("```json").or_else(|| s.strip_prefix("```")) {
        if let Some(inner) = inner.strip_suffix("```") {
            return inner.trim();
        }
    }
    s
}

pub async fn call_ai(
    provider: &str,
    api_key: Option<&str>,
    model: &str,
    base_url: Option<&str>,
    prompt: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    match provider {
        "claude" => {
            let key = api_key
                .filter(|k| !k.is_empty())
                .ok_or("Anthropic API key is required for Claude")?;
            let req = ClaudeRequest {
                model: model.to_string(),
                max_tokens: 1024,
                messages: vec![AiMessage { role: "user".to_string(), content: prompt }],
            };
            let res: ClaudeResponse = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&req)
                .send().await
                .map_err(|e| format!("Claude request failed: {e}"))?
                .json().await
                .map_err(|e| format!("Claude response parse failed: {e}"))?;
            res.content.into_iter()
                .find(|c| c.content_type == "text")
                .and_then(|c| c.text)
                .ok_or_else(|| "Empty response from Claude".to_string())
        }

        "openai" => {
            let key = api_key
                .filter(|k| !k.is_empty())
                .ok_or("OpenAI API key is required")?;
            let url = base_url
                .filter(|u| !u.is_empty())
                .unwrap_or("https://api.openai.com/v1/chat/completions");
            let req = OpenAiRequest {
                model: model.to_string(),
                max_tokens: 1024,
                messages: vec![AiMessage { role: "user".to_string(), content: prompt }],
            };
            let res: OpenAiResponse = client
                .post(url)
                .header("Authorization", format!("Bearer {key}"))
                .header("content-type", "application/json")
                .json(&req)
                .send().await
                .map_err(|e| format!("OpenAI request failed: {e}"))?
                .json().await
                .map_err(|e| format!("OpenAI response parse failed: {e}"))?;
            res.choices.into_iter()
                .next()
                .map(|c| c.message.content)
                .ok_or_else(|| "Empty response from OpenAI".to_string())
        }

        "ollama" => {
            let base = base_url
                .filter(|u| !u.is_empty())
                .unwrap_or("http://localhost:11434");
            let url = format!("{}/v1/chat/completions", base.trim_end_matches('/'));
            let req = OpenAiRequest {
                model: model.to_string(),
                max_tokens: 1024,
                messages: vec![AiMessage { role: "user".to_string(), content: prompt }],
            };
            let res: OpenAiResponse = client
                .post(url)
                .header("content-type", "application/json")
                .json(&req)
                .send().await
                .map_err(|e| format!("Ollama request failed — is it running? {e}"))?
                .json().await
                .map_err(|e| format!("Ollama response parse failed: {e}"))?;
            res.choices.into_iter()
                .next()
                .map(|c| c.message.content)
                .ok_or_else(|| "Empty response from Ollama".to_string())
        }

        other => Err(format!("Unknown provider '{other}'")),
    }
}
