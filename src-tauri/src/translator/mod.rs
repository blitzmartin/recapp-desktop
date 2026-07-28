pub mod ollama;

pub trait Translator {
    async fn translate(
        &self,
        text: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<String, String>;
}
