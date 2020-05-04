from setuptools import setup, find_packages
from os import path

here = path.abspath(path.dirname(__file__))

# # Get the long description from the README file
# try:
#     with open(path.join(here, 'README.md'), encoding='utf-8') as f:
#         long_description = f.read()
# except IOError:
#     long_description = ""

setup(
    name="polar",  # Required
    version="0.0.0",  # Required
    description="polar python package",  # Optional
    author="oso",  # Optional
    classifiers=[  # Optional
        "Development Status :: 3 - Alpha",
        "Programming Language :: Python :: 3.6",
    ],
    packages=find_packages(),  # Required
    python_requires=">=3.6",
    setup_requires=["cffi>=1.0.0"],
    cffi_modules=["polar/build.py:ffibuilder"],
    install_requires=["cffi>=1.0.0"],
    extras_require={},  # Optional
    package_data={"polar": ["policies/*.pol", "policies/*.polar"]},  # Optional
    entry_points={"console_scripts": [],},  # Optional
    project_urls={},  # Optional
)