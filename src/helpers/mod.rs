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

pub trait SplitExt {
    /// Split at pattern, only if not inside an open parentheses
    fn split_checked(self, pattern: char) -> Vec<Self>
    where
        Self: Sized;
}

impl SplitExt for &str {
    fn split_checked(self, pattern: char) -> Vec<Self> {
        let mut parentheses = 0;
        let mut start = 0;
        let mut result = vec![];
        let mut reset = false;
        for (i, c) in self.char_indices() {
            if reset {
                start = i;
                reset = false;
            }
            match c {
                '(' => {
                    parentheses += 1;
                }
                ')' => {
                    parentheses -= 1;
                }
                c if c == pattern => {
                    if parentheses == 0 {
                        result.push(self[start..i].trim());
                        reset = true;
                    }
                }
                _ => {}
            }
        }
        if !reset {
            result.push(self[start..].trim());
        }
        result
    }
}
