pub trait UnEscapeString {
    /// Unescape special characters like '\n' and '\t'
    fn unescape(self) -> Option<String>;
}

impl UnEscapeString for Option<&Option<String>> {
    fn unescape(self) -> Option<String> {
        self.and_then(|s| {
            s.as_ref()
                .map(|s| s.replace(r"\n", "\n").replace(r"\t", "\t"))
        })
    }
}
