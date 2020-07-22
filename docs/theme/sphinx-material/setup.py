from setuptools import setup

import versioneer

with open("requirements.txt") as reqs:
    REQUIREMENTS = [reqs.readlines()]

with open("requirements-dev.txt") as dev_reqs:
    REQUIREMENTS_DEV = [dev_reqs.readlines()]

setup(
    name="sphinx_material",
    version=versioneer.get_version(),
    cmdclass=versioneer.get_cmdclass(),
    description="Material sphinx theme",
    long_description=open("README.rst").read(),
    author="Kevin Sheppard",
    author_email="kevin.k.sheppard@gmail.com",
    url="https://github.com/bashtage/sphinx-material",
    packages=["sphinx_material"],
    include_package_data=True,
    python_requires=">=3.6",
    install_requires=REQUIREMENTS,
    extras_require={"dev": REQUIREMENTS_DEV},
    license="MIT",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Natural Language :: English",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python",
        "Framework :: Sphinx :: Extension",
        "Framework :: Sphinx :: Theme",
        "Topic :: Documentation :: Sphinx",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
    ],
    entry_points={"sphinx.html_themes": ["sphinx_material = sphinx_material",]},
)
