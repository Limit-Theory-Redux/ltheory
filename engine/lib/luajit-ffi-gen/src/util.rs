/// Convert snake case string into a camel case one.
///
/// Rules:
/// - make first letter an upper case
/// - remove underscore and make after that the first letter upper case
/// - make first letter after digit an upper case
pub fn snake_to_camel_case(s: &str, first_upper: bool) -> String {
    let mut res = String::new();
    let mut to_upper = first_upper;

    for c in s.trim_start_matches('_').chars() {
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

/// Convert camel case string into a snake case one.
pub fn camel_to_snake_case(s: &str, to_upper: bool) -> String {
    let mut res = String::new();
    let mut need_underscore = false;

    for c in s.chars() {
        if c == '_' {
            res.push(c);
            need_underscore = false;
        } else if c.is_digit(10) {
            res.push(c);
            need_underscore = true;
        } else if c.is_uppercase() {
            if need_underscore {
                res.push('_');
            } else {
                need_underscore = true;
            }

            if to_upper {
                res.push(c);
            } else {
                res += &c.to_lowercase().to_string();
            }
        } else {
            if to_upper {
                res += &c.to_uppercase().to_string();
            } else {
                res.push(c);
            }
            need_underscore = true;
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel_case1() {
        let res = snake_to_camel_case("set_var", true);
        assert_eq!(res, "SetVar")
    }

    #[test]
    fn test_snake_to_camel_case2() {
        let res = snake_to_camel_case("set2d", true);
        assert_eq!(res, "Set2D")
    }

    #[test]
    fn test_snake_to_camel_case3() {
        let res = snake_to_camel_case("set_2d", true);
        assert_eq!(res, "Set2D")
    }

    #[test]
    fn test_snake_to_camel_case4() {
        let res = snake_to_camel_case("set_var", false);
        assert_eq!(res, "setVar")
    }

    #[test]
    fn test_snake_to_camel_case5() {
        let res = snake_to_camel_case("_set_var", false);
        assert_eq!(res, "setVar")
    }

    #[test]
    fn test_camel_to_snake_case1() {
        let res = camel_to_snake_case("SetVar", false);
        assert_eq!(res, "set_var")
    }

    #[test]
    fn test_camel_to_snake_case2() {
        let res = camel_to_snake_case("SetVar", true);
        assert_eq!(res, "SET_VAR")
    }

    #[test]
    fn test_camel_to_snake_case3() {
        let res = camel_to_snake_case("_SetVar", false);
        assert_eq!(res, "_set_var")
    }

    #[test]
    fn test_camel_to_snake_case4() {
        let res = camel_to_snake_case("_Set_Var", false);
        assert_eq!(res, "_set_var")
    }
}
