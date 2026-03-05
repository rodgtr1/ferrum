use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }

    pub fn cycle_next(&self) -> HttpMethod {
        match self {
            HttpMethod::GET => HttpMethod::POST,
            HttpMethod::POST => HttpMethod::PUT,
            HttpMethod::PUT => HttpMethod::PATCH,
            HttpMethod::PATCH => HttpMethod::DELETE,
            HttpMethod::DELETE => HttpMethod::HEAD,
            HttpMethod::HEAD => HttpMethod::OPTIONS,
            HttpMethod::OPTIONS => HttpMethod::GET,
        }
    }

    pub fn cycle_prev(&self) -> HttpMethod {
        match self {
            HttpMethod::GET => HttpMethod::OPTIONS,
            HttpMethod::POST => HttpMethod::GET,
            HttpMethod::PUT => HttpMethod::POST,
            HttpMethod::PATCH => HttpMethod::PUT,
            HttpMethod::DELETE => HttpMethod::PATCH,
            HttpMethod::HEAD => HttpMethod::DELETE,
            HttpMethod::OPTIONS => HttpMethod::HEAD,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl KeyValuePair {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum AuthConfig {
    #[default]
    None,
    BearerToken { token: String },
    BasicAuth { username: String, password: String },
    ApiKey { header: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<KeyValuePair>,
    pub query_params: Vec<KeyValuePair>,
    pub body: String,
    pub auth: AuthConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_as_str() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn test_method_cycle() {
        assert_eq!(HttpMethod::GET.cycle_next(), HttpMethod::POST);
        assert_eq!(HttpMethod::OPTIONS.cycle_next(), HttpMethod::GET);
        assert_eq!(HttpMethod::GET.cycle_prev(), HttpMethod::OPTIONS);
    }
}
