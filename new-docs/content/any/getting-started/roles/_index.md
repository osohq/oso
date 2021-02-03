---
title: Add roles
weight: 2
description: >
  Roles are an intuitive way to group permissions and expose them to end
  users. Use oso to add roles to your application.
---

# Introduction to Roles

<!-- TODO: Review me  -->

**Roles** are a way to group permissions together and assign them to users.

A well-designed role system has roles that map to an intuitive concept of what
a user should be able to do in the application.

<!-- -- Insert image along this lines of
[this](https://slides.com/samscott/access-python#/13/0/0) --  -->

For example, as the "Owner" of a document in Google Drive, I can invite someone
as a "Viewer", "Commenter", or "Editor". As a user of the application, it is
clear from each of these names what I can expect to be able to do. There are
still some cases that might not be obvious (can viewers invite others to
view?), but for the primary use cases the roles correspond cleanly to
permissions.

A small number of roles goes a long way: I can also assign a user to a role for
an entire folder. Now they will _inherit_ this role for all files and
subfolders.

Keep reading to learn more about roles or head over to [Role-Based Access
Control](learn/roles) to learn about how roles work in oso and about RBAC
design patterns.
