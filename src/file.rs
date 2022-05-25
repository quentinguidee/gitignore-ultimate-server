use ropey::Rope;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

pub struct File {
    pub url: Url,
    pub text: Rope,
}

impl File {
    pub fn new(url: Url, text: String) -> Self {
        File {
            url,
            text: Rope::from_str(text.as_str()),
        }
    }

    pub fn apply_change(&mut self, change: TextDocumentContentChangeEvent) {
        let text = change.text;
        let range = match change.range {
            Some(range) => range,
            None => return,
        };

        let start = range.start;
        let start = self.text.line_to_char(start.line as usize) + (start.character as usize);

        let end = range.end;
        let end = self.text.line_to_char(end.line as usize) + end.character as usize;

        self.text.remove(start..end);
        self.text.insert(start, text.as_str());
    }
}
