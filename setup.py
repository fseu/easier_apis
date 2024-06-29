from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    rust_extensions=[RustExtension("easier_apis.easier_apis_core", binding=Binding.NoBinding)],
)
