# EasierAPIs 🚀

EasierAPIs is a Python framework that simplifies the process of interacting with RESTful APIs. It provides a clean, intuitive interface for making API requests, with built-in support for authentication, caching, and middleware. Powered by a Rust core, EasierAPIs offers high performance while maintaining a user-friendly Python API.

## ✨ Features

    - 🧘‍♂️ Simple and minimalistic syntax for defining API endpoints
    - 🔐 Built-in authentication methods (Bearer, Basic, and Custom)
    - 🔄 Request/response middleware support
    - 🔁 Automatic retries for failed requests
    - 💾 In-memory caching with time-based expiration
    - 🦀 Powered by Rust for high performance
    - 🎨 Decorator-based routing for easy endpoint definition
    - 🔧 Support for GET, POST, PUT, and DELETE methods

## 📦 Installation

You can install EasierAPIs using pip:

```bash
pip install easier-apis
```

Note: This package requires a Rust compiler to be installed on your system for the initial setup.

## 🚀 Quick Start

Here's a simple example of how to use EasierAPIs:

```python
from easier_apis import API, get, post

api = API("https://api.example.com")

# Set authentication (if required)
api.set_auth("Bearer", value="your_token_here")

@api.route("/users/{id}")
@get
def get_user(id: int):
    return {}

@api.route("/users")
@post
def create_user(name: str, email: str):
    return {"name": name, "email": email}

# Usage
user = get_user(1, cache_ttl=60)  # Cache for 60 seconds
new_user = create_user("John Doe", "john@example.com")
```

## 📘 Detailed Usage

### 🏗️ Initializing the API

```python
from easier_apis import API

api = API("https://api.example.com")
```

### 🔐 Setting Authentication

```python
# Bearer token
api.set_auth("Bearer", value="your_token_here")

# Basic auth
api.set_auth("Basic", key="username", value="password")

# Custom auth
api.set_auth("Custom", key="X-API-Key", value="your_api_key_here")
```

### 🛣️ Defining Endpoints

Use decorators to define your API endpoints:

```python
@api.route("/users")
@get
def get_users():
    return {}

@api.route("/users/{id}")
@get
def get_user(id: int):
    return {}

@api.route("/users")
@post
def create_user(name: str, email: str):
    return {"name": name, "email": email}

@api.route("/users/{id}")
@put
def update_user(id: int, name: str, email: str):
    return {"id": id, "name": name, "email": email}

@api.route("/users/{id}")
@delete
def delete_user(id: int):
    return {}
```

### 💾 Using Caching

You can enable caching for GET requests by specifying a cache_ttl:

```python
user = get_user(1, cache_ttl=60)  # Cache for 60 seconds
```

To invalidate the cache:

```python
# Invalidate a specific endpoint
api.invalidate_cache("/users/1")

# Invalidate entire cache
api.invalidate_cache()
```

### 🔄 Adding Middleware

You can add middleware to modify requests or responses:

```python
def log_request(data):
    print(f"Outgoing request data: {data}")
    return data

api.add_middleware(log_request)
```

### 🔬 Advanced Usage

### 🚨 Custom Error Handling

EasierAPIs uses Rust for core operations, which provides robust error handling. You can catch and handle these errors in your Python code:

```python
try:
    user = get_user(1)
except Exception as e:
    print(f"An error occurred: {e}")
```

### ⚙️ Configuring Retries

The Rust core automatically retries failed requests. The current implementation retries up to 3 times with exponential backoff. This behavior can be customized by modifying the Rust code if needed.

### 🤝 Contributing

Contributions to EasierAPIs are welcome Please feel free to submit a Pull Request.

### 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.

### 🙏 Acknowledgments

This project uses reqwest for HTTP requests in Rust.
The Python-Rust integration is handled using CFFI.

### 🆘 Support

If you encounter any problems or have any questions, please open an issue on the GitHub repository.
