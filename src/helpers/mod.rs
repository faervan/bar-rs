use std::collections::HashMap;

use regex::Regex;

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

pub trait ParseTemplate {
    fn parse_template(&self, template: &str) -> String;
}

impl ParseTemplate for HashMap<String, Box<dyn ToString + Send + Sync>> {
    fn parse_template(&self, template: &str) -> String {
        let regex = Regex::new(r"\{\{(.*?)\}\}").unwrap();
        regex
            .replace_all(template, |caps: &regex::Captures| {
                let key = &caps[1];
                self.get(key)
                    .map_or_else(|| format!("{{{{{}}}}}", key), |v| v.to_string())
            })
            .to_string()
    }
}

/// Create a `HashMap<String, Box<dyn ToString + Send + Sync>>` from a list of ("key", "value")
/// tuples
macro_rules! create_map {
    ( $( ($key:expr, $value:expr) ),* ) => {{
        let mut map = HashMap::new();
        $(
            map.insert(
                $key.to_string(),
                Box::new($value) as Box<dyn ToString + Send + Sync>,
            );
        )*
        map
    }};
}
