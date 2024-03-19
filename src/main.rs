pub mod engines;
pub mod normalize;
pub mod parse;
pub mod web;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    unsafe {
        const ADDITIONAL_PROFANE_WORDS: &[&str] =
            &["furry", "bomb", "kill", "murder", "nsfw", "dox"];
        for word in ADDITIONAL_PROFANE_WORDS {
            rustrict::Trie::customize_default().set(word, rustrict::Type::INAPPROPRIATE);
        }
    }

    web::run().await;
}
