package com.osohq.oso;

import java.io.IOException;
import java.util.*;
import java.util.function.Function;

public class Oso {
    Polar polar;

    public Oso() throws Exceptions.OsoException {
        polar = new Polar();
        registerClass(Http.class,
                (m) -> new Http((String) m.get("hostname"), (String) m.get("path"), (String) m.get("query")), "Http");
        registerClass(PathMapper.class, (m) -> new PathMapper((String) m.get("template")), "PathMapper");
    }

    /**
     * Enqueue a polar policy file to be loaded. File contents are loaded into a
     * String and saved here, so changes to the file made after calls to loadFile
     * will not be recognized. If the filename already exists in the load queue,
     * replace it.
     *
     * @param filename
     * @throws Exceptions.PolarFileExtensionError On incorrect file extension.
     * @throws IOException                        If unable to open or read the
     *                                            file.
     */
    public void loadFile(String filename) throws Exceptions.PolarFileExtensionError, IOException {
        polar.loadFile(filename);
    }

    /**
     * Clear the KB, but maintain all registered classes and calls
     *
     * @throws Exceptions.OsoException
     */
    public void clear() throws Exceptions.OsoException {
        polar.clear();
    }

    /**
     * Load a Polar string into the KB (without filename).
     *
     * @param str Polar string to be loaded.
     * @throws Exceptions.OsoException
     */
    public void loadStr(String str) throws Exceptions.OsoException {
        polar.loadStr(str);
    }

    /**
     * Load a Polar string into the KB (with filename).
     *
     * @param str      Polar string to be loaded.
     * @param filename Name of the source file.
     * @throws Exceptions.OsoException
     */
    public void loadStr(String str, String filename) throws Exceptions.OsoException {
        polar.loadStr(str, filename);
    }

    /**
     * Register a Java class with oso.
     *
     * @param cls       Class object to be registered.
     * @param fromPolar lambda function to convert from a
     *                  {@code Map<String, Object>} of parameters to an instance of
     *                  the Java class.
     * @throws Exceptions.DuplicateClassAliasError if class has already been
     *                                             registered.
     */
    public void registerClass(Class cls, Function<Map, Object> fromPolar) throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
        polar.registerClass(cls, fromPolar);
    }

    /**
     * Register a Java class with oso using an alias.
     *
     * @param cls       Class object to be registered.
     * @param fromPolar lambda function to convert from a
     *                  {@code Map<String, Object>} of parameters to an instance of
     *                  the Java class.
     * @param alias     name to register the class under, which is how the class is
     *                  accessed from Polar.
     * @throws Exceptions.DuplicateClassAliasError if a class has already been
     *                                             registered with the given alias.
     */
    public void registerClass(Class cls, Function<Map, Object> fromPolar, String alias)
            throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
        polar.registerClass(cls, fromPolar, alias);
    }

    /**
     * Submit an `allow` query to the Polar knowledge base.
     *
     * @param actor
     * @param action
     * @param resource
     * @return
     * @throws Exceptions.OsoException
     */
    public boolean allow(Object actor, Object action, Object resource) throws Exceptions.OsoException {
        return polar.queryPred("allow", List.of(actor, action, resource)).hasMoreElements();
    }

    /**
     * Query for a Predicate.
     *
     * @param name Predicate name, e.g. "f" for predicate "f(x)".
     * @param args List of predicate arguments.
     * @return List of resulting variable bindings.
     * @throws Exceptions.OsoException
     */
    public List<HashMap<String, Object>> queryPredicate(String name, List<Object> args) throws Exceptions.OsoException {
        return polar.queryPred(name, args).results();
    }

    /**
     * Start the Polar REPL.
     *
     * @throws Exceptions.OsoException
     */
    public void repl() throws Exceptions.OsoException {
        polar.repl();
    }
}