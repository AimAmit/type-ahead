pub trait WordImpl {
    // fn new(id: u32) -> Self;
    fn update_pos(&mut self, doc_id: u32, pos: u32);
}

pub trait WordProcesImpl {
    fn get_or_create_word_mut(&mut self, word: &str) -> &mut dyn WordImpl;
}
