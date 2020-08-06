package com.osohq.oso;

import java.lang.reflect.Constructor;
import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.*;
import java.util.function.Function;

import com.osohq.oso.Exceptions.ParseError;
import com.osohq.oso.Exceptions.PolarRuntimeException;

public class Polar {
    private Ffi.Polar ffiPolar;
    protected Host host; // visible for tests only
    private Map<String, String> loadQueue; // Map from filename -> file contents

    public Polar() throws Exceptions.OsoException {
        ffiPolar = Ffi.get().polarNew();
        host = new Host(ffiPolar);
        loadQueue = new HashMap<String, String>();

        // Register built-in classes.
        registerClass(Boolean.class, "Boolean");
        registerClass(Integer.class, "Integer");
        registerClass(Double.class, "Float");
        registerClass(List.class, "List");
        registerClass(Map.class, "Dictionary");
        registerClass(String.class, "String");
    }

    /**
     * Clear the KB, but maintain all registered classes and calls.
     *
     * @throws Exceptions.OsoException
     */
    public void clear() throws Exceptions.OsoException {
        loadQueue.clear();
        ffiPolar = Ffi.get().polarNew();
    }

    /**
     * Enqueue a polar policy file to be loaded. File contents are loaded into a
     * String and saved here, so changes to the file made after calls to loadFile
     * will not be recognized. If the filename already exists in the load queue,
     * replace it.
     *
     * @throws Exceptions.PolarFileExtensionError On incorrect file extension.
     * @throws IOException                        If unable to open or read the file.
     */
    public void loadFile(String filename) throws IOException, Exceptions.PolarFileExtensionError {
        Optional<String> ext = Optional.ofNullable(filename).filter(f -> f.contains("."))
                .map(f -> f.substring(filename.lastIndexOf(".") + 1));

        // check file extension
        if (!ext.isPresent() || !ext.get().equals("polar")) {
            throw new Exceptions.PolarFileExtensionError();
        }

        // add file to queue
        loadQueue.put(filename, new String(Files.readAllBytes(Paths.get(filename))));
    }

    /**
     * Load a Polar string into the KB (with filename).
     *
     * @param str      Polar string to be loaded.
     * @param filename Name of the source file.
     */
    public void loadStr(String str, String filename) throws Exceptions.OsoException {
        ffiPolar.loadStr(str, filename);
        checkInlineQueries();
    }

    /**
     * Load a Polar string into the KB (without filename).
     *
     * @param str Polar string to be loaded.
     */
    public void loadStr(String str) throws Exceptions.OsoException {
        ffiPolar.loadStr(str, null);
        checkInlineQueries();
    }

    /**
     * Query for a predicate, parsing it first.
     */
    public Query query(String query) throws Exceptions.OsoException {
        loadQueuedFiles();
        return new Query(ffiPolar.newQueryFromStr(query), host.clone());
    }

    /**
     * Query for a predicate.
     */
    public Query query(Predicate query) throws Exceptions.OsoException {
        loadQueuedFiles();
        Host new_host = host.clone();
        String pred = new_host.toPolarTerm(query).toString();
        return new Query(ffiPolar.newQueryFromTerm(pred), new_host);
    }

    /**
     * Query for a rule.
     *
     * @param rule Rule name, e.g. "f" for rule "f(x)".
     * @param args Variable list of rule arguments.
     */
    public Query queryRule(String rule, Object... args) throws Exceptions.OsoException {
        loadQueuedFiles();
        Host new_host = host.clone();
        String pred = new_host.toPolarTerm(new Predicate(rule, Arrays.asList(args))).toString();
        return new Query(ffiPolar.newQueryFromTerm(pred), new_host);
    }

    /**
     * Start the Polar REPL.
     */
    public void repl() throws Exceptions.OsoException, IOException {
        repl(new String[0]);
    }

    /**
     * Load the given files and start the Polar REPL.
     */
    public void repl(String[] files) throws Exceptions.OsoException, IOException {
        for (String file : files) {
            loadFile(file);
        }
        loadQueuedFiles();

        BufferedReader in = new BufferedReader(new InputStreamReader(System.in));
        Ffi.Query ffiQuery;
        Query query;
        String input;
        while (true) {
            System.out.print("query> ");
            input = in.readLine();
            if (input == null) {
                return;
            }
            for (int n = input.length() - 1; n > 0 && input.charAt(n) == ';'; n--) {
                input = input.substring(0, n);
            }

            try {
                ffiQuery = ffiPolar.newQueryFromStr(input);
            } catch (ParseError e) {
                System.out.println("Parse error: " + e.toString());
                continue;
            }

            try {
                query = new Query(ffiQuery, host);
            } catch (PolarRuntimeException e) {
                System.out.println(e.toString());
                continue;
            }

            if (!query.hasMoreElements()) {
                System.out.println("false");
            } else {
                do {
                    HashMap<String, Object> result = query.nextElement();
                    System.out.println(result.size() > 0 ? result.toString() : "true");
                } while (query.hasMoreElements());
            }
        }
    }

    public static void main(String[] args) throws Exceptions.OsoException, IOException {
        new Polar().repl(args);
    }

    /**
     * Register a Java class with Polar.
     */
    public void registerClass(Class<?> cls)
            throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
        registerClass(cls, cls.getName(), null);
    }

    /**
     * Register a Java class with Polar using a specific constructor.
     */
    public void registerClass(Class<?> cls, Constructor<?> constructor)
            throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
        registerClass(cls, cls.getName(), constructor);
    }

    /**
     * Register a Java class with Polar using an alias.
     */
    public void registerClass(Class<?> cls, String name)
            throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
        registerClass(cls, name, null);
    }

    /**
     * Register a Java class with an optional constructor and alias.
     */
    public void registerClass(Class<?> cls, String name, Constructor<?> constructor)
            throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
        host.cacheClass(cls, constructor, name);
        registerConstant(name, cls);
    }

    /**
     * Registers `value` as a Polar constant variable called `name`.
     */
    public void registerConstant(String name, Object value) throws Exceptions.OsoException {
        ffiPolar.registerConstant(name, host.toPolarTerm(value).toString());
    }

    /**
     * Load all queued files, flushing the {@code loadQueue}
     */
    private void loadQueuedFiles() throws Exceptions.OsoException {
        for (String fname : loadQueue.keySet()) {
            loadStr(loadQueue.get(fname), fname);
        }
        loadQueue.clear();
    }

    /**
     * Confirm that all queued inline queries succeed.
     */
    private void checkInlineQueries() throws Exceptions.OsoException, Exceptions.InlineQueryFailedError {
        Ffi.Query nextQuery = ffiPolar.nextInlineQuery();
        while (nextQuery != null) {
            if (!new Query(nextQuery, host).hasMoreElements()) {
                throw new Exceptions.InlineQueryFailedError();
            }
            nextQuery = ffiPolar.nextInlineQuery();
        }
    }
}
