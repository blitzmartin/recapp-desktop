pub mod ollama;

pub trait LlmProvider {
    async fn summarize(&self, text: &str, language: &str, num_words: u32) -> Result<String, String>;
}
