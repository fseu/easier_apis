use reqwest::{Client, Request, Response};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use std::time::Duration;
use std::sync::Arc;
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

pub struct RustCore {
    client: Client,
    base_url: String,
    auth: Option<Auth>,
    middleware: Vec<Arc<dyn Fn(Request) -> Request + Send + Sync>>,
}

pub enum Auth {
    Bearer(String),
    Basic(String, String),
    Custom(String, String),
}

impl RustCore {
    pub fn new(base_url: &str) -> Self {
        RustCore {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            base_url: base_url.to_string(),
            auth: None,
            middleware: Vec::new(),
        }
    }

    pub fn set_auth(&mut self, auth: Auth) {
        self.auth = Some(auth);
    }

    pub fn add_middleware<F>(&mut self, middleware: F)
    where
        F: Fn(Request) -> Request + Send + Sync + 'static,
    {
        self.middleware.push(Arc::new(middleware));
    }

    fn apply_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth {
            Some(Auth::Bearer(token)) => request.header(AUTHORIZATION, format!("Bearer {}", token)),
            Some(Auth::Basic(username, password)) => request.basic_auth(username, Some(password)),
            Some(Auth::Custom(key, value)) => request.header(key, value),
            None => request,
        }
    }

    fn apply_middleware(&self, mut request: Request) -> Request {
        for middleware in &self.middleware {
            request = middleware(request);
        }
        request
    }

    pub fn fetch(&self, path: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        let request = self.client.get(&url);
        let request = self.apply_auth(request);
        let request = request.build()?;
        let request = self.apply_middleware(request);
        
        let response = self.send_with_retry(request)?;
        let json: Value = response.json()?;
        Ok(json)
    }

    pub fn send(&self, path: &str, method: &str, data: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.base_url, path);
        let request = match method {
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            _ => return Err("Unsupported method".into()),
        };
        let request = self.apply_auth(request);
        let request = request.json(&data).build()?;
        let request = self.apply_middleware(request);
        
        let response = self.send_with_retry(request)?;
        let json: Value = response.json()?;
        Ok(json)
    }

    fn send_with_retry(&self, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        let mut attempts = 0;
        let max_attempts = 3;
        
        loop {
            match self.client.execute(request.try_clone().unwrap()) {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() && attempts < max_attempts {
                        attempts += 1;
                        std::thread::sleep(Duration::from_secs(2u64.pow(attempts)));
                        continue;
                    } else {
                        return Err(format!("HTTP error: {}", response.status()).into());
                    }
                }
                Err(e) if attempts < max_attempts => {
                    attempts += 1;
                    std::thread::sleep(Duration::from_secs(2u64.pow(attempts)));
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_core_new(base_url: *const c_char) -> *mut RustCore {
    let c_str = unsafe { CStr::from_ptr(base_url) };
    let base_url = c_str.to_str().unwrap();
    Box::into_raw(Box::new(RustCore::new(base_url)))
}

#[no_mangle]
pub extern "C" fn rust_core_fetch(core: *mut RustCore, path: *const c_char) -> *mut c_char {
    let core = unsafe { &*core };
    let c_str = unsafe { CStr::from_ptr(path) };
    let path = c_str.to_str().unwrap();
    
    match core.fetch(path) {
        Ok(json) => CString::new(json.to_string()).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rust_core_send(core: *mut RustCore, path: *const c_char, method: *const c_char, data: *const c_char) -> *mut c_char {
    let core = unsafe { &*core };
    let c_path = unsafe { CStr::from_ptr(path) };
    let path = c_path.to_str().unwrap();
    let c_method = unsafe { CStr::from_ptr(method) };
    let method = c_method.to_str().unwrap();
    let c_data = unsafe { CStr::from_ptr(data) };
    let data: Value = serde_json::from_str(c_data.to_str().unwrap()).unwrap();
    
    match core.send(path, method, data) {
        Ok(json) => CString::new(json.to_string()).unwrap().into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn rust_core_free(ptr: *mut c_char) {
    unsafe {
        if ptr.is_null() { return }
        CString::from_raw(ptr)
    };
}

#[no_mangle]
pub extern "C" fn rust_core_set_auth(core: *mut RustCore, auth_type: *const c_char, key: *const c_char, value: *const c_char) {
    let core = unsafe { &mut *core };
    let c_auth_type = unsafe { CStr::from_ptr(auth_type) };
    let auth_type = c_auth_type.to_str().unwrap();
    let c_key = unsafe { CStr::from_ptr(key) };
    let key = c_key.to_str().unwrap();
    let c_value = unsafe { CStr::from_ptr(value) };
    let value = c_value.to_str().unwrap();

    let auth = match auth_type {
        "Bearer" => Auth::Bearer(value.to_string()),
        "Basic" => Auth::Basic(key.to_string(), value.to_string()),
        "Custom" => Auth::Custom(key.to_string(), value.to_string()),
        _ => return,
    };

    core.set_auth(auth);
}
