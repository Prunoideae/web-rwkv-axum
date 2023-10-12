from setuptools import setup, find_packages

setup(
    name="web-rwkv-axum",
    version="0.1",
    packages=["web_rwkv_axum", "web_rwkv_axum.components"],
    install_requires=["websockets", "ujson"],
    include_package_data=True,
)
