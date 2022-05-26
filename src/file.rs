use ropey::Rope;
use std::path::Path;
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

    pub fn get_line_content(&self, line_number: u32) -> String {
        self.text.line(line_number as usize).to_string()
    }

    pub fn path(&self) -> &Path {
        Path::new(self.url.path())
    }

    pub fn directory(&self) -> Option<&Path> {
        let parent = self.path().parent()?;
        Some(parent)
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

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use crate::file::File;

    #[test]
    fn it_can_get_line_content() {
        let url = Url::parse("file:///a").unwrap();
        let content = "First line\nSecond line\nThird line".to_string();

        let file = File::new(url, content);

        let line_content = file.get_line_content(1);
        assert_eq!(line_content, "Second line\n");
    }

    #[test]
    fn it_can_get_path() {
        let url = Url::parse("file:///Users/me/file").unwrap();
        let file = File::new(url, "".to_string());

        let line_content = file.path().to_str().unwrap();
        assert_eq!(line_content, "/Users/me/file");
    }

    #[test]
    fn it_can_get_directory() {
        let url = Url::parse("file:///Users/me/file").unwrap();
        let file = File::new(url, "".to_string());

        let directory = file.directory().unwrap();
        assert_eq!(directory.to_str().unwrap(), "/Users/me");
    }
}
