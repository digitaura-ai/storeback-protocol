from setuptools import setup, find_packages

setup(
    name="storeback",
    version="1.0.0",
    description="Official Python SDK for the Storeback Protocol (RFC-0402 Autonomous Machine-to-Machine Knowledge Vending)",
    author="Storeback Protocol Working Group",
    license="Apache-2.0",
    packages=find_packages(),
    install_requires=[
        "requests>=2.28.0",
    ],
    python_requires=">=3.9",
)
