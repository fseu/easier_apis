import functools
from typing import Callable, Any, Dict
import json
from cffi import FFI
import time
from collections import OrderedDict

ffi = FFI()
ffi.cdef("""
    void* rust_core_new(const char* base_url);
    char* rust_core_fetch(void* core, const char* path);
    char* rust_core_send(void* core, const char* path, const char* method, const char* data);
    void rust_core_free(char* ptr);
    void rust_core_set_auth(void* core, const char* auth_type, const char* key, const char* value);
""")
lib = ffi.dlopen("libeasier_apis_core.so")  # Adjust path as needed

class LRUCache:
    def __init__(self, capacity: int = 100):
        self.cache = OrderedDict()
        self.capacity = capacity

    def get(self, key: str) -> Any:
        if key not in self.cache:
            return None
        value, expiry = self.cache[key]
        if expiry and time.time() > expiry:
            del self.cache[key]
            return None
        self.cache.move_to_end(key)
        return value

    def put(self, key: str, value: Any, ttl: int = None):
        if key in self.cache:
            del self.cache[key]
        elif len(self.cache) >= self.capacity:
            self.cache.popitem(last=False)
        expiry = time.time() + ttl if ttl else None
        self.cache[key] = (value, expiry)
        self.cache.move_to_end(key)

class API:
    def __init__(self, base_url: str, cache_capacity: int = 100):
        self.base_url = base_url
        self.rust_core = lib.rust_core_new(base_url.encode('utf-8'))
        self.middleware = []
        self.cache = LRUCache(cache_capacity)

    def set_auth(self, auth_type: str, key: str = "", value: str = ""):
        lib.rust_core_set_auth(self.rust_core, auth_type.encode('utf-8'), key.encode('utf-8'), value.encode('utf-8'))

    def add_middleware(self, middleware: Callable[[Dict[str, Any]], Dict[str, Any]]):
        self.middleware.append(middleware)

    def route(self, path: str):
        def decorator(func: Callable):
            @functools.wraps(func)
            def wrapper(*args, **kwargs):
                return func(*args, **kwargs)
            return wrapper
        return decorator

    def fetch(self, path: str, cache_ttl: int = None) -> Dict[str, Any]:
        cache_key = f"GET:{path}"
        cached_data = self.cache.get(cache_key)
        if cached_data:
            return cached_data

        result = lib.rust_core_fetch(self.rust_core, path.encode('utf-8'))
        json_str = ffi.string(result).decode('utf-8')
        lib.rust_core_free(result)
        data = json.loads(json_str)
        data = self._apply_middleware(data)

        if cache_ttl is not None:
            self.cache.put(cache_key, data, cache_ttl)

        return data

    def send(self, path: str, method: str, data: Dict[str, Any]) -> Dict[str, Any]:
        data = self._apply_middleware(data)
        json_data = json.dumps(data)
        result = lib.rust_core_send(self.rust_core, path.encode('utf-8'), method.encode('utf-8'), json_data.encode('utf-8'))
        json_str = ffi.string(result).decode('utf-8')
        lib.rust_core_free(result)
        return json.loads(json_str)

    def _apply_middleware(self, data: Dict[str, Any]) -> Dict[str, Any]:
        for middleware in self.middleware:
            data = middleware(data)
        return data

    def invalidate_cache(self, path: str = None):
        if path:
            cache_key = f"GET:{path}"
            if cache_key in self.cache.cache:
                del self.cache.cache[cache_key]
        else:
            self.cache.cache.clear()

def get(func: Callable) -> Callable:
    @functools.wraps(func)
    def wrapper(self, *args, **kwargs):
        path = self.route.__closure__[0].cell_contents
        cache_ttl = kwargs.pop('cache_ttl', None)
        formatted_path = path.format(*args, **kwargs)
        return self.fetch(formatted_path, cache_ttl)
    return wrapper

def post(func: Callable) -> Callable:
    @functools.wraps(func)
    def wrapper(self, *args, **kwargs):
        path = self.route.__closure__[0].cell_contents
        data = func(*args, **kwargs)
        formatted_path = path.format(*args, **kwargs)
        return self.send(formatted_path, "POST", data)
    return wrapper

def put(func: Callable) -> Callable:
    @functools.wraps(func)
    def wrapper(self, *args, **kwargs):
        path = self.route.__closure__[0].cell_contents
        data = func(*args, **kwargs)
        formatted_path = path.format(*args, **kwargs)
        return self.send(formatted_path, "PUT", data)
    return wrapper

def delete(func: Callable) -> Callable:
    @functools.wraps(func)
    def wrapper(self, *args, **kwargs):
        path = self.route.__closure__[0].cell_contents
        formatted_path = path.format(*args, **kwargs)
        return self.fetch(formatted_path)
    return wrapper
