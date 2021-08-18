use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Headers = HashMap<String, Vec<String>>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Option<Headers>,
    pub body: Option<String>,
}

impl Request {
    pub fn get(url: &str) -> Request {
        Request {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: None,
            body: None,
        }
    }

    pub fn body(self, body: &str) -> Request {
        Request {
            method: self.method,
            url: self.url,
            headers: self.headers,
            body: Some(body.to_string()),
        }
    }

    pub fn set(self, name: &str, key: &str) -> Request {
        let headers = match self.headers {
            Some(mut headers) => {
                if let Some(header) = headers.get_mut(name) {
                    header.push(key.to_string());
                } else {
                    headers.insert(name.to_string(), vec![key.to_string()]);
                }

                Some(headers)
            }
            None => None,
        };

        Request {
            method: self.method,
            url: self.url,
            headers,
            body: self.body,
        }
    }

    pub fn call(self) -> Response {
        http_request(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub headers: Headers,
    pub body: String,
    pub status: i32,
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
pub fn http_request(req: Request) -> Response {
    if let Err(err) = tanoshi_lib::shim::write_object(req) {
        return Response {
            headers: HashMap::new(),
            body: format!("{}", err),
            status: 9999,
        };
    }

    unsafe { host_http_request() };
    tanoshi_lib::shim::read_object().unwrap_or_else(|err| Response {
        headers: HashMap::new(),
        body: format!("{}", err),
        status: 9999,
    })
}

#[cfg(all(not(feature = "__test"), not(feature = "host")))]
#[link(wasm_import_module = "tanoshi")]
extern "C" {
    fn host_http_request();
}

#[cfg(any(feature = "__test", feature = "host"))]
pub fn http_request(req: Request) -> Response {
    use std::str::FromStr;

    use log::debug;
    use reqwest::Method;

    let client = match reqwest::blocking::Client::builder()
        .user_agent("Tanoshi/0.1.0")
        .build()
    {
        Ok(method) => method,
        Err(e) => {
            return Response {
                headers: HashMap::new(),
                body: format!("failed to build client: {}", e),
                status: 9999,
            };
        }
    };

    let method = match Method::from_str(&req.method) {
        Ok(method) => method,
        Err(e) => {
            return Response {
                headers: HashMap::new(),
                body: format!("{} requests are not valid: {}", req.method, e),
                status: 9999,
            };
        }
    };

    let mut request_builder = client.request(method, req.url);
    if let Some(headers) = req.headers.as_ref() {
        for (key, values) in headers {
            for value in values {
                request_builder = request_builder.header(key, value);
            }
        }
    }

    if let Some(body) = req.body {
        request_builder = request_builder.body(body);
    }

    debug!("request => {:?}", request_builder);

    match request_builder.send() {
        Ok(response) => {
            debug!("response ok => {:?}", response);

            let status = response.status();
            Response {
                headers: {
                    let header_map = response.headers();
                    header_map
                        .keys()
                        .map(|key| {
                            (
                                key.to_string(),
                                header_map
                                    .get_all(key)
                                    .iter()
                                    .flat_map(|value| value.to_str().ok().map(str::to_string))
                                    .collect(),
                            )
                        })
                        .collect()
                },
                body: response.text().unwrap_or_else(|_| "".to_string()),
                status: status.as_u16() as i32,
            }
        }
        Err(err) => {
            debug!("response error => {:?}", err);

            Response {
                headers: HashMap::new(),
                body: format!("{}", err),
                status: 9999,
            }
        }
    }
}
