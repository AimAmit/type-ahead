use super::WordProcesImpl;

pub trait DocumentImpl {
    fn process<T: WordProcesImpl>(&self, word_map: &mut T);
}

pub trait DocumentMapImpl {
    fn add_doc(&mut self, text: String) -> impl DocumentImpl;
}