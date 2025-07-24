use anyhow::Result;
use mockall::automock;

/// A trait that abstracts the behavior of an LLM provider.
#[automock] // This will automatically generate MockLlmProvider
pub trait LlmProvider {
    fn generate(&self, prompt: &str) -> Result<String>;
}
