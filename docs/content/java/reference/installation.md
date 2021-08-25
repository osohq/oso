---
title: Installation
weight: 1
description: Installation instructions for Oso's Java library.
aliases: 
  - ./lib.html
---

# Installation

The Java version of Oso is available on [Maven
Central](https://search.maven.org/artifact/com.osohq/oso).

It can be added as a dependency to a **Maven** project:

```xml
<!-- https://mvnrepository.com/artifact/com.osohq/oso -->
<dependency>
    <groupId>com.osohq</groupId>
    <artifactId>oso</artifactId>
    <version>{{< version >}}</version>
</dependency>
```

or a **Gradle** project:

```gradle
// https://mvnrepository.com/artifact/com.osohq/oso
compile group: 'com.osohq', name: 'oso', version: '{relase}'
```

or downloaded as a **JAR** and added to the classpath of any Java project:

```console
$ javac -classpath "oso-{{< version >}}.jar:." MyProject.java

$ java -classpath "oso-{{< version >}}.jar:." MyProject
```

## Requirements

- Java version 10 or greater
- Supported platforms:
  - Linux
  - macOS
  - Windows
