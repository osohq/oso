.. meta::
   :description: Download the latest release of oso, an open source policy engine for authorization.

:orphan:

============
Installation
============

oso is available as a library in several languages:

.. tabs::
    .. group-tab:: Python

        The Python version of oso is available on `PyPI`_ and can be installed using
        ``pip``::

            $ pip install oso=={release}

        To install Python framework integrations, see:

        - :doc:`/using/frameworks/flask`
        - :doc:`/using/frameworks/django`
        - :doc:`/using/frameworks/sqlalchemy`

        For more information on the oso Python library, see the
        :doc:`library documentation </using/libraries/python/index>`.

        .. admonition:: What's next
            :class: tip

            After you've installed oso, check out the
            :doc:`/getting-started/quickstart`.

        **Requirements**

        - Python version 3.6 or greater
        - Supported platforms:
            - Linux
            - OS X
            - Windows

        The Python version is known to work on glibc-based distributions but not on musl-based ones
        (like Alpine Linux).  Wheels built against musl that you can use on
        Alpine Linux can be downloaded from `the releases page on GitHub
        <https://github.com/osohq/oso/releases/latest>`_.

        .. _PyPI: https://pypi.org/project/oso/


    .. group-tab:: Ruby

        The Ruby version of oso is available on RubyGems_ and can be installed
        into your local Ruby::

            $ gem install oso-oso -v {release}

        or added to a project's ``Gemfile``::

            gem 'oso-oso', '~> {release}'

        For more information on the oso Ruby library, see the
        :doc:`library documentation </using/libraries/ruby/index>`.

        .. admonition:: What's next
            :class: tip

            After you've installed oso, check out the
            :doc:`/getting-started/quickstart`.

        **Requirements**

        - Ruby version 2.4 or greater
        - Supported platforms:
            - Linux
            - OS X
            - Windows

        .. _RubyGems: https://rubygems.org/gems/oso-oso


    .. group-tab:: Java

        The Java version of oso is available on `Maven Central <https://search.maven.org/artifact/com.osohq/oso>`_.

        It can be added as a dependency to a **Maven** project::

            <!-- https://mvnrepository.com/artifact/com.osohq/oso -->
            <dependency>
                <groupId>com.osohq</groupId>
                <artifactId>oso</artifactId>
                <version>{release}</version>
            </dependency>

        or a **Gradle** project::

            // https://mvnrepository.com/artifact/com.osohq/oso
            compile group: 'com.osohq', name: 'oso', version: '{relase}'

        or downloaded as a **JAR** and added to the classpath of any Java project::

            $ javac -classpath "{JAR}:." MyProject.java

            $ java -classpath "{JAR}:." MyProject

        For more information on the oso Java library, see the
        :doc:`library documentation </using/libraries/java/index>`.

        .. admonition:: What's next
            :class: tip

            After you've installed oso, check out the
            :doc:`/getting-started/quickstart`.

        **Requirements**

        - Java version 10 or greater
        - Supported platforms:
            - Linux
            - OS X
            - Windows

    .. group-tab:: Node.js

        The Node.js version of oso is available on NPM_ and can be installed
        globally with NPM::

            $ npm install -g oso@{release}

        or added as a dependency to a project's ``package.json`` manifest with
        NPM::

            $ npm install oso@{release}

        or Yarn::

            $ yarn add oso@{release}

        For more information on the oso Node.js library, see the :doc:`library
        documentation </using/libraries/node/index>`.

        .. admonition:: What's next
            :class: tip

            After you've installed oso, check out the
            :doc:`/getting-started/quickstart`.

        **Requirements**

        - Node.js version 10 or greater
        - Supported platforms:
            - Linux
            - OS X
            - Windows

        .. _NPM: https://www.npmjs.com/package/oso

    .. group-tab:: Rust

        The rust version of oso is available on crates.io_.

        Add oso and oso-derive as dependencies in your Cargo.toml

        .. code-block:: toml

            oso = "{release}"
            oso-derive = "{release}"

        For more information on the oso Rust library, see the
        :doc:`library documentation </using/libraries/rust/index>`.

        .. admonition:: What's next
            :class: tip

            After you've installed oso, check out the
            :doc:`/getting-started/quickstart`.

        **Requirements**

        - Rust stable
        - Supported platforms:
            - Linux
            - OS X
            - Windows

        .. _crates.io: https://crates.io/crates/oso




**Libraries coming soon:**

- Go

Source Code
-----------

The source code for oso is hosted on GitHub:

:fab:`github` `osohq/oso <https://github.com/osohq/oso>`_


Releases
--------
.. toctree::
    :maxdepth: 1
    :caption: See below for release notes:

    changelogs/2021-02-03
    changelogs/2021-01-20
    changelogs/2020-12-22
    changelogs/2020-12-08
    changelogs/2020-11-25
    changelogs/2020-11-10
    changelogs/2020-10-27
    changelogs/2020-10-13
    changelogs/2020-09-22
    changelogs/2020-09-08
    changelogs/2020-09-01
    changelogs/2020-08-25
    changelogs/2020-08-11
    changelogs/2020-07-28
    changelogs/2020-07-14
    changelogs/2020-06-30
    changelogs/2020-06-03
    changelogs/2020-05-11
    changelogs/2020-04-24

.. include:: /newsletter.rst
