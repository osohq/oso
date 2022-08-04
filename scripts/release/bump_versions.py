"""Bump versions in files for release.

Requirements:

    - tomlkit: Format preserving TOML parser.
"""

import pathlib
import sys
import re
import argparse
import fileinput

from packaging.version import parse as parse_version
import tomlkit

"""The base oso directory."""
BASE = pathlib.Path(__file__).parent.parent

"""Regular expression for capturing version."""
VERSION_RE = r"[\w.]+"


def log(*args):
    print(*args, file=sys.stderr)


def replace_version(target_version, path, match_re=None):
    """Set version to ``target_version`` in ``path``.

    :param match_re: If provided, replace capture group 1 that matches match_re
    """
    if match_re is None:
        log(f"{path}: {target_version}")
        with open(path, 'w') as f:
            f.write(target_version + "\n")
        return

    match_re = re.compile(match_re)
    with fileinput.FileInput(files=(path.as_posix(),), inplace=True) as f:
        match_found = False
        for line in f:
            match = match_re.search(line)
            if match is not None:
                match_found = True
                start, end = match.span(1)

                replace_line_with = line
                replace_line_with = (replace_line_with[:start] + target_version
                                     + replace_line_with[end:])
                log(f"{path}: {line.strip()} => {replace_line_with.strip()}")

                # In file input context, stdout writes to the file.
                sys.stdout.write(replace_line_with)
            else:
                sys.stdout.write(line)

    assert match_found, f"Match not found for {path}"


def replace_version_toml(filename, mutations):
    """Apply ``mutations`` to TOML formatted ``filename``.

    :param mutations: Mutations is a dictionary describing keys and values
                      in the TOML file to update. Keys are specified as a ``.``
                      separated path.
    """
    with open(filename, "r") as f:
        contents_str = f.read()
        contents = tomlkit.parse(contents_str)

    for (path, update) in mutations.items():
        parts = path.split(".")
        o = contents
        for part in parts[:-1]:
            o = o[part]

        log(f"{filename}: {path} => {update}")
        o[parts[-1]] = update

    with open(filename, "w") as f:
        write_str = tomlkit.dumps(contents)
        f.write(write_str)


def bump_oso_version(version):
    replace_version(version, BASE / "VERSION")
    replace_version(version,
                    BASE / "languages/java/oso/pom.xml",
                    fr"<!-- oso_version --><version>({VERSION_RE})<\/version>")
    replace_version(version,
                    BASE / "docs/examples/Makefile",
                    fr"JAVA_PACKAGE_JAR_PATH := .*\/oso-({VERSION_RE})\.jar")
    replace_version(version,
                    BASE / "languages/js/package.json",
                    fr'"version": "({VERSION_RE})"')
    replace_version(version,
                    BASE / "languages/python/docs/conf.py",
                    fr'version = "({VERSION_RE})"')
    replace_version(version,
                    BASE / "languages/python/docs/conf.py",
                    fr'release = "({VERSION_RE})"')
    replace_version(version,
                    BASE / "languages/python/oso/oso/oso.py",
                    fr'__version__ = "({VERSION_RE})"')
    replace_version(version,
                    BASE / "languages/ruby/Gemfile.lock",
                    fr'oso-oso \(({VERSION_RE})\)')
    replace_version(version,
                    BASE / "languages/ruby/lib/oso/version.rb",
                    fr"VERSION = '({VERSION_RE})'")
    replace_version_toml(BASE / "languages/rust/oso-derive/Cargo.toml",
                         {
                             "package.version": version
                         })
    replace_version_toml(BASE / "languages/rust/oso/Cargo.toml",
                         {
                             "package.version": version,
                             "dependencies.oso-derive.version": f"={version}",
                             "dependencies.polar-core.version": f"={version}",
                             "dev-dependencies.oso-derive.version": f"={version}",
                         })
    replace_version_toml(BASE / "polar-c-api/Cargo.toml",
                         {
                             "package.version": version,
                             "dependencies.polar-core.version": f"={version}",
                         })
    replace_version_toml(BASE / "polar-core/Cargo.toml",
                         {
                             "package.version": version,
                         })
    replace_version_toml(BASE / "polar-wasm-api/Cargo.toml",
                         {
                             "package.version": version,
                             "dependencies.polar-core.version": f"={version}",
                         })
    replace_version_toml(BASE / "polar-language-server/Cargo.toml",
                         {
                             "package.version": version,
                             "dependencies.polar-core.version": f"={version}",
                         })
    replace_version(version,
                    BASE / ".github/workflows/publish-docs.yml",
                    fr'default: "({VERSION_RE})" # oso_version')
    replace_version(version,
                    BASE / "vscode/oso/package.json",
                    fr'"version": "({VERSION_RE})"')


def oso_python_dependency_version(version):
    """Get oso version that Python dependencies should pin to.

    0.14.5 => 0.14.0
    """
    parsed = parse_version(version)
    return ".".join((str(parsed.major), str(parsed.minor), str(0)))


def bump_sqlalchemy_version(version, oso_version):
    replace_version(version,
                    BASE / "languages/python/sqlalchemy-oso/sqlalchemy_oso/__init__.py",
                    fr'__version__ = "({VERSION_RE})"')
    replace_version(oso_python_dependency_version(oso_version),
                    BASE / "languages/python/sqlalchemy-oso/requirements.txt",
                    fr'oso~=({VERSION_RE})')
    replace_version(version,
                    BASE / ".github/workflows/publish-docs.yml",
                    fr'default: "({VERSION_RE})" # sqlalchemy_oso_version')


def bump_flask_version(version, oso_version):
    replace_version(version,
                    BASE / "languages/python/flask-oso/flask_oso/__init__.py",
                    fr'__version__ = "({VERSION_RE})"')
    replace_version(oso_python_dependency_version(oso_version),
                    BASE / "languages/python/flask-oso/requirements.txt",
                    fr'oso~=({VERSION_RE})')
    replace_version(version,
                    BASE / ".github/workflows/publish-docs.yml",
                    fr'default: "({VERSION_RE})" # flask_oso_version')


def bump_django_version(version, oso_version):
    replace_version(version,
                    BASE / "languages/python/django-oso/django_oso/__init__.py",
                    fr'__version__ = "({VERSION_RE})"')
    replace_version(oso_python_dependency_version(oso_version),
                    BASE / "languages/python/django-oso/requirements.txt",
                    fr'oso~=({VERSION_RE})')
    replace_version(version,
                    BASE / ".github/workflows/publish-docs.yml",
                    fr'default: "({VERSION_RE})" # django_oso_version')



def bump_versions(oso_version=None, sqlalchemy_version=None,
                  flask_version=None, django_version=None):
    if oso_version is not None:
        bump_oso_version(oso_version)

    if sqlalchemy_version is not None:
        assert oso_version is not None, "--oso_version must be provided"
        bump_sqlalchemy_version(sqlalchemy_version, oso_version)

    if flask_version is not None:
        assert oso_version is not None, "--oso_version must be provided"
        bump_flask_version(flask_version, oso_version)

    if django_version is not None:
        assert oso_version is not None, "--oso_version must be provided"
        bump_django_version(django_version, oso_version)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--oso_version')
    parser.add_argument('--sqlalchemy_version')
    parser.add_argument('--flask_version')
    parser.add_argument('--django_version')

    bump_versions(**vars(parser.parse_args()))


if __name__ == '__main__':
    main()
