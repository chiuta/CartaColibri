//! Presets for the chat services people already know. Almost all of them speak the same
//! OpenAI-style HTTP as the local engines, so one name is enough to connect (`--provider mistral`).
//! Local engines stay the default; a cloud service is used only when you say so (`--allow-cloud`
//! or COLI_ALLOW_CLOUD=1) because the passages sent with a question leave your computer.

use crate::carta::Backend;

pub struct Provider {
    pub name: &'static str,
    pub label: &'static str,
    pub url: &'static str,
    /// Environment variable that holds the key (besides COLI_API_KEY); empty for local engines.
    pub key_env: &'static str,
    pub cloud: bool,
    pub note: &'static str,
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        name: "colibri",
        label: "Colibrì (reference engine)",
        url: "http://127.0.0.1:8000",
        key_env: "",
        cloud: false,
        note: "local, default",
    },
    Provider {
        name: "ollama",
        label: "Ollama",
        url: "http://127.0.0.1:11434",
        key_env: "",
        cloud: false,
        note: "local; `ollama pull` a chat and an embedding model",
    },
    Provider {
        name: "lmstudio",
        label: "LM Studio",
        url: "http://127.0.0.1:1234",
        key_env: "",
        cloud: false,
        note: "local; start its server, load a model",
    },
    Provider {
        name: "mistral",
        label: "Mistral (Le Chat)",
        url: "https://api.mistral.ai/v1",
        key_env: "MISTRAL_API_KEY",
        cloud: true,
        note: "key from console.mistral.ai",
    },
    Provider {
        name: "openai",
        label: "OpenAI (ChatGPT)",
        url: "https://api.openai.com/v1",
        key_env: "OPENAI_API_KEY",
        cloud: true,
        note: "key from platform.openai.com",
    },
    Provider {
        name: "gemini",
        label: "Google Gemini",
        url: "https://generativelanguage.googleapis.com/v1beta/openai",
        key_env: "GEMINI_API_KEY",
        cloud: true,
        note: "key from aistudio.google.com; compatibility layer is in beta",
    },
    Provider {
        name: "cohere",
        label: "Cohere (Coral)",
        url: "https://api.cohere.ai/compatibility/v1",
        key_env: "COHERE_API_KEY",
        cloud: true,
        note: "key from dashboard.cohere.com",
    },
    Provider {
        name: "nvidia",
        label: "NVIDIA build (Nemotron, Llama)",
        url: "https://integrate.api.nvidia.com/v1",
        key_env: "NVIDIA_API_KEY",
        cloud: true,
        note: "free credits at build.nvidia.com",
    },
    Provider {
        name: "claude",
        label: "Claude (Anthropic)",
        url: "https://api.anthropic.com/v1",
        key_env: "ANTHROPIC_API_KEY",
        cloud: true,
        note: "OpenAI-compatible layer, meant for testing and comparison; embeddings not offered",
    },
    Provider {
        name: "meta",
        label: "Meta AI (Llama API)",
        url: "https://api.llama.com/compat/v1",
        key_env: "LLAMA_API_KEY",
        cloud: true,
        note: "Llama API access is by invitation in some regions",
    },
];

/// Services with no public HTTP API: only their website exists, so they cannot be connected.
pub const WEB_ONLY: &[&str] = &[
    "Microsoft Copilot (copilot.microsoft.com)",
    "the Meta AI website (meta.ai) and the ChatGPT / Gemini / Le Chat / Claude web pages themselves: use the API names above",
];

pub fn find(name: &str) -> Option<&'static Provider> {
    let n = name.trim().to_lowercase();
    let n = match n.as_str() {
        "chatgpt" | "gpt" => "openai",
        "le-chat" | "lechat" | "mistralai" => "mistral",
        "google" => "gemini",
        "coral" => "cohere",
        "nemotron" | "nim" => "nvidia",
        "anthropic" => "claude",
        "llama" => "meta",
        "coli" | "local" => "colibri",
        "lm-studio" => "lmstudio",
        x => x,
    };
    PROVIDERS.iter().find(|p| p.name == n)
}

pub fn key_of(p: &Provider) -> Option<String> {
    if p.key_env.is_empty() {
        return None;
    }
    std::env::var(p.key_env)
        .ok()
        .filter(|k| !k.trim().is_empty())
}

pub fn cloud_allowed(flag: bool) -> bool {
    flag || std::env::var("COLI_ALLOW_CLOUD")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// A backend for `spec`: `NAME` or `NAME:MODEL` (for example `mistral:mistral-small-latest`, `ollama:llama3.2`).
/// Cloud services need `allow_cloud`, a key in their environment variable (or COLI_API_KEY) and a model.
pub fn backend(spec: &str, model: Option<&str>, allow_cloud: bool) -> Result<Backend, String> {
    let (name, m) = match spec.split_once(':') {
        Some((n, m)) if !m.is_empty() => (n, Some(m.to_string())),
        _ => (spec, model.map(String::from)),
    };
    let p = find(name).ok_or_else(|| {
        format!(
            "unknown provider `{name}`. Known: {}. Run `cartacolibri providers`.",
            PROVIDERS
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    if p.cloud {
        if !cloud_allowed(allow_cloud) {
            return Err(format!(
                "{} is a cloud service: the question and the passages found in your documents would be sent to it. \
                 Add --allow-cloud (or set COLI_ALLOW_CLOUD=1) if that is what you want.",
                p.label
            ));
        }
        if m.is_none() {
            return Err(format!(
                "{} needs a model name: use `--provider {}:MODEL` or --model MODEL (see the service's model list)",
                p.label, p.name
            ));
        }
    }
    let mut be = Backend::gateway(p.url, m);
    // context packs are a corbul feature: never offered to another service
    be.packs = false;
    if p.cloud {
        // never send another service's key to a cloud host by accident: only its own variable or COLI_API_KEY
        be.api_key = key_of(p).or_else(|| {
            std::env::var("COLI_API_KEY")
                .ok()
                .filter(|k| !k.trim().is_empty())
        });
        if be.api_key.is_none() {
            return Err(format!("{} needs a key: set {}", p.label, p.key_env));
        }
    } else {
        be.api_key = std::env::var("COLI_API_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
            .or(be.api_key);
    }
    Ok(be)
}
