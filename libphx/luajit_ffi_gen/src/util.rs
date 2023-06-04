pub fn as_camel_case(s: &str) -> String {
    let mut res = String::new();
    let mut to_upper = true;

    // TODO: do we need special treatment if string starts with '_'?
    for c in s.chars() {
        if c == '_' {
            // Skip underscores
            to_upper = true;
        } else if c.is_digit(10) {
            res.push(c);
            // First letter after numbers should be uppercase
            to_upper = true;
        } else if to_upper {
            res += &c.to_uppercase().to_string();
            to_upper = false;
        } else {
            res.push(c);
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::as_camel_case;

    #[test]
    fn test_as_camel_case1() {
        let res = as_camel_case("set_var");
        assert_eq!(res, "SetVar")
    }

    #[test]
    fn test_as_camel_case2() {
        let res = as_camel_case("set2d");
        assert_eq!(res, "Set2D")
    }
}
