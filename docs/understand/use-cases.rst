Use Cases
=========

..todo::
    Move to its own (sidebar) section

    The three areas:
    - Customer-facing applications
    - Internal applications
    - Infrastructure

Authorization is a broad subject, and arises in many different areas. Although oso
is designed to be used anywhere, and for any type of authorization, there are some
applications that are more naturally suited for it.

Currently, the *ideal* use case for oso is within multi-tenant applications
with some degree of complexity in the permissions scheme. For example, any application
where authorization decisions need to take into account *who* the user is and their
*relation* to the data being accessed.

For applications where all users fall into two roles (e.g. admin and user), the value
in using oso might be to enable moving to a more complex model at a later date.

oso *does not* handle assigning users to roles, or assigning permissions to users directly. Although you *can* do this with oso, our belief is that this data is better managed by the application in whatever database is already in place. oso can be used to
reference that data directly, express what roles can do in an application, and even extend the roles to include inheritance structures and hierarchies.

This means that currently oso should not be seen as a replacement for things like AWS IAM or Active Directory. In the future, these may be possible, and if you ever want someone to rant to about these kinds of things, you'll find us happy to listen.
