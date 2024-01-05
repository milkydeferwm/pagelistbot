//! A toy template engine.

/// Apply the template, insert necessary context values into the template.
pub fn apply_template(template: &str, cx: &std::collections::HashMap<char, String>) -> String {
    let mut out = String::new();
    let mut escape = false;
    for ch in template.chars() {
        match ch {
            '$' => if escape {
                out.push('$');
                escape = false;
            } else {
                escape = true;
            }
            ch => if escape {
                if let Some(t) = cx.get(&ch) {
                    out.push_str(t);
                } else {
                    out.push('$');
                    out.push(ch);
                }
                escape = false;
            } else {
                out.push(ch);
            }
        }
    }
    if escape {
        out.push('$');
    }
    out
}

#[macro_export]
macro_rules! context {
    () => {
        std::collections::HashMap::new()
    };
    ($($k:expr=>$v:expr),+ $(,)?) => {
        std::collections::HashMap::from([$(($k,$v)),+])
    };
}

#[cfg(test)]
mod test {
    use crate::{apply_template, context};

    #[test]
    fn test_macro() {
        let map = context![
            'c' => String::from("CCC"),
            'f' => String::from("FFF"),
        ];
        assert_eq!(map, std::collections::HashMap::from_iter([
            ('c', String::from("CCC")),
            ('f', String::from("FFF"))
        ]));
    }

    #[test]
    fn test_macro_empty() {
        let map: std::collections::HashMap<char, String> = context![];
        assert_eq!(map, std::collections::HashMap::new());
    }

    #[test]
    fn test_template() {
        let s = apply_template("abcabc$$$kc$cc$", &std::collections::HashMap::from_iter([
            ('k', String::from("hahaha")),
        ]));
        assert_eq!(s, "abcabc$hahahac$cc$");
    }

    #[test]
    fn test_template_with_macro() {
        let s = apply_template("abcabc$$$kc$cc$", &context![
            'k' => String::from("hahaha"),
        ]);
        assert_eq!(s, "abcabc$hahahac$cc$");
    }
}
