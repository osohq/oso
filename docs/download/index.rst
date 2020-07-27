============
Installation
============

oso is available as a library in several languages:

.. tabs::
    .. group-tab:: Python

        The Python version of oso is available on `PyPI`_ and can be installed using
        ``pip``::

            $ pip install oso

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
            - Windows (coming soon)

        .. _PyPI: https://pypi.org/project/oso/


    .. group-tab:: Ruby

        The Ruby version of oso is available on RubyGems_ and can be installed using
        ``gem``::

            $ gem install oso-oso

        or added to your ``Gemfile``::

            gem 'oso-oso', '~> 0.2.2'

        and installed with `bundler`_.

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
            - Windows (coming soon)

        .. _RubyGems: https://rubygems.org/gems/oso-oso
        .. _bundler: https://bundler.io/


    .. group-tab:: Java

        The Java version of oso is available on GitHub. Go to the `Maven Repository <https://github.com/osohq/oso/packages/321403>`_ and download the latest jar.

        To use it, add it to the classpath for your project::

            $ javac -classpath "oso-|release|.jar:." MyProject.java

            $ java -classpath "oso-|release|.jar:." MyProject

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
            - Windows (coming soon)

**Libraries Coming soon**

- Go
- JavaScript
- Rust

Source code
-----------

The source code for oso is hosted on GitHub.

.. todo::
  Link to GitHub


Releases
--------
.. toctree::
    :maxdepth: 1
    :caption: See below for release notes:

    v0.2.0 <changelogs/0.2.0>
    v0.1.0 <changelogs/0.1.0>
    v0.0.4 <changelogs/0.0.4>
    v0.0.3 <changelogs/0.0.3>
    v0.0.2 <changelogs/0.0.2>

.. include:: /newsletter.rst
