---
title: Deprecating the Oso Open Source Library
weight: 1
any: true
hideContents: false
draft: false
---

# Deprecating the Oso Open Source Library

**Date: 2023/12/18**

Today we’re deprecating the legacy Oso open source library. We have plans for the next open source release and we’re looking forward to getting feedback from the community leading up to that point. In the meantime, if you’re happy using the Oso open source library now, nothing needs to change – i.e., we are not end-of-lifing (EOL) the library and we’ll continue to provide support and critical bug fixes.

This post describes how we got here, what this change means for existing users, and what you can expect from Oso in the future. If you have questions, you can always reach out to us in our community Slack.

### How we got here

We started working on the Oso library in 2020. We believed that Polar could help developers with a piece of the authorization problem, and we turned out to be right. We worked with thousands of users to make Polar more intuitive and to solve some of the thornier problems in authorization, like data filtering.

Through this process, we learned a ton about what was good and not so good about the Oso library. Polar – this was good! We have continued to lean into this. But over time we continued to see two main challenges with the implementation of the library:

1. API boundary
   1. The Oso library centers around the [Foreign Function Interface (FFI)](https://doc.rust-lang.org/nomicon/ffi.html).
   2. While convenient in some ways, this abstraction has proved to be more confusing than useful.
   3. The upshot is a fuzzy API surface area for Oso, which created footguns for our users in setup and when debugging.
2. Performance
   1. Since the Oso library doesn’t store any data itself, it relies on your existing model (by definition)
   2. This led to unpredictable performance, especially for data filtering use cases
   3. The hooks into ORMs (for `sqlalchemy-oso` and `django-oso`) made it easier to set up the Oso library, but querying by way of the ORM also contributed to these performance challenges

Based on feedback from our users, we decided to build Oso Cloud to solve a set of problems that the library doesn’t solve – authorization for microservices. When we built Oso cloud, we wanted to apply what we’ve learned from the library’s API boundary and performance issues. This is what gave rise to the Facts API and Facts data model, respectively. To make this happen, we had a choice: try to refactor the existing library, or start from a clean slate. We chose the latter, which enabled us to leave our technical debt behind. This created a new challenge: ever since that point, we’ve had 2 codebases, 2 sets of libraries, and 2 documentation sites.

This is neither good for our users nor good for us. It’s made it harder for us to maintain the Oso library, to build new features for it, and to support it. We want to fix that, but it’s going to take some time.

### What we’re doing and what it means for users

Our first step is to deprecate the Oso library. We’ve also moved the Oso library documentation to https://www.osohq.com/docs/oss. The implications for most existing users are: nothing. That is, if the Oso library is working for you, there’s no action required at the moment.

Our plan is to start open sourcing core components of the latest Oso implementation from here. This will take time – we plan to do this over multiple releases. While the first things we open source will not be suitable for all use cases, we’re confident that the core architectural changes will be well worth it for the developer community. And over time, we plan to reach use case parity with what exists today.

In the meantime, we’re not going anywhere. We’re committed to making critical bug fixes and providing best-efforts support if you’re having issues. We’re not planning to end of life (EOL) the Oso library for at least 12 months. And once we have a suitable replacement for the Oso library, we’ll provide documentation on how to migrate, as well as make ourselves available via Slack and Zoom, as always.

### Oso’s open source future

We know that deprecating software you’re using is inconvenient at best, but we believe this step is the best way to set up Oso and the broader development community for the long-term.

We believe in the power of open source. This is the first step towards delivering a better and more sustainable open source Oso. While we aren’t sharing specifics on those just yet, we’re happy to share more details and hear your feedback 1x1. In particular, if there are areas you’d be interested to learn about and/or contribute, we’d love to hear it! And more generally, if you have any questions or feedback, feel free to reach out to us in Slack.
