use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceType {
    syntax: Syntax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Syntax {
    Sass,
    Scss,
    Css,
}

impl Default for SourceType {
    fn default() -> Self {
        Self { syntax: Syntax::Scss }
    }
}

impl SourceType {
    pub fn from_path<P: AsRef<Path>>(&self, path: P) -> Self {
        let syntax = match path.as_ref().extension() {
            Some(ext) if ext == "css" => Syntax::Css,
            Some(ext) if ext == "sass" => Syntax::Sass,
            _ => Syntax::Scss,
        };

        Self { syntax }
    }
}
