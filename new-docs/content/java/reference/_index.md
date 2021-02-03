---
title: Reference
weight: 4
any: false

---
# Java Authorization Library

The Java version of Oso is available on [Maven Central](https://search.maven.org/artifact/com.osohq/oso).

It can be added as a dependency to a **Maven** project:

```
<!-- https://mvnrepository.com/artifact/com.osohq/oso -->
<dependency>
    <groupId>com.osohq</groupId>
    <artifactId>oso</artifactId>
    <version>0.9.0</version>
</dependency>
```

or a **Gradle** project:

```
// https://mvnrepository.com/artifact/com.osohq/oso
compile group: 'com.osohq', name: 'oso', version: '{relase}'
```

or downloaded as a **JAR** and added to the classpath of any Java project:

```
$ javac -classpath "oso-0.9.0.jar:." MyProject.java

$ java -classpath "oso-0.9.0.jar:." MyProject
```

For more information on the Oso Java library, see the
library documentation.

**Requirements**

* Java version 10 or greater
* Supported platforms:
  * Linux
  * OS X
  * Windows
