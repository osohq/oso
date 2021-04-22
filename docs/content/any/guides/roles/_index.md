---
title: Add Roles
weight: 1
description: >
  Roles are an intuitive way to group permissions and expose them to end
  users. Use Oso to add roles to your application.
aliases:
  - roles/index.html
---

# Add Roles to Your App

<!-- TODO: Review me  -->

**Roles** are a way to group permissions together and assign them to users.
These guides will help you add roles to your application and use them to perform authorization using Oso.

To learn more about roles and the different ways they can be used for authorization, head over to [Role-Based Access
Control](learn/roles).

{{% callout "Implementing roles?" "yellow" %}}
We're working on the next major feature of Oso, a new set of tools to better support adding roles to B2B applications.
These features include defining roles, adding role checks/enforcement to your application, and exposing a
role management API and frontend to your end users.

If you're interested, we encourage you to sign up for early access [here](https://osohq.typeform.com/to/w8xgMHbw)!
{{% /callout %}}

<!-- A well-designed role system has roles that map to an intuitive concept of what
a user should be able to do in the application. -->

<!-- -- Insert image along this lines of
[this](https://slides.com/samscott/access-python#/13/0/0) --  -->

<!-- For example, as the "Owner" of a document in Google Drive, I can invite someone
as a "Viewer", "Commenter", or "Editor". As a user of the application, it is
clear from each of these names what I can expect to be able to do. There are
still some cases that might not be obvious (can viewers invite others to
view?), but for the primary use cases the roles correspond cleanly to
permissions. -->

<!-- A small number of roles goes a long way: I can also assign a user to a role for
an entire folder. Now they will _inherit_ this role for all files and
subfolders. -->

<!-- Keep reading to learn more about roles or head over to [Role-Based Access
Control](learn/roles) to learn about how roles work in Oso and about RBAC
design patterns. -->
