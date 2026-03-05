use crate::models::environment::Environment;

/// Replace all `{{KEY}}` placeholders with values from the active environment.
pub fn interpolate(input: &str, env: Option<&Environment>) -> String {
    let Some(env) = env else { return input.to_string() };
    let mut result = input.to_string();
    for var in &env.vars {
        if !var.enabled {
            continue;
        }
        let placeholder = format!("{{{{{}}}}}", var.key);
        result = result.replace(&placeholder, &var.value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::environment::{Environment, EnvVar};

    fn make_env() -> Environment {
        let mut env = Environment::new("test");
        env.vars.push(EnvVar::new("BASE_URL", "https://example.com"));
        env.vars.push(EnvVar::new("TOKEN", "abc123"));
        env
    }

    #[test]
    fn test_basic_interpolation() {
        let env = make_env();
        let result = interpolate("{{BASE_URL}}/users", Some(&env));
        assert_eq!(result, "https://example.com/users");
    }

    #[test]
    fn test_multiple_vars() {
        let env = make_env();
        let result = interpolate("{{BASE_URL}}/auth?token={{TOKEN}}", Some(&env));
        assert_eq!(result, "https://example.com/auth?token=abc123");
    }

    #[test]
    fn test_no_env() {
        let result = interpolate("{{BASE_URL}}/users", None);
        assert_eq!(result, "{{BASE_URL}}/users");
    }

    #[test]
    fn test_unknown_var_unchanged() {
        let env = make_env();
        let result = interpolate("{{UNKNOWN}}/path", Some(&env));
        assert_eq!(result, "{{UNKNOWN}}/path");
    }

    #[test]
    fn test_disabled_var_unchanged() {
        let mut env = make_env();
        env.vars[0].enabled = false;
        let result = interpolate("{{BASE_URL}}/path", Some(&env));
        assert_eq!(result, "{{BASE_URL}}/path");
    }
}
