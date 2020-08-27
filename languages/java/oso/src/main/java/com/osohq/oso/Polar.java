package com.osohq.oso;

import java.lang.reflect.Constructor;
import java.io.BufferedReader;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.*;

import com.osohq.oso.Exceptions.OsoException;
import com.osohq.oso.Exceptions.ParseError;
import com.osohq.oso.Exceptions.PolarRuntimeException;

public class Polar {
    private Ffi.Polar ffiPolar;
    protected Host host; // visible for tests only
    private Map<String, String> loadedNames; // Map from filename -> file contents
    private Map<String, String> loadedContent; // Map from file contents -> filename

    public Polar() throws Exceptions.OsoException {
        ffiPolar = Ffi.get().polarNew();
        host = new Host(ffiPolar);
        loadedNames = new HashMap<String, String>();
        loadedContent = new HashMap<String, String>();

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
        loadedNames.clear();
        loadedContent.clear();
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
    public void loadFile(String filename) throws IOException, OsoException {
        Optional<String> ext = Optional.ofNullable(filename).filter(f -> f.contains("."))
                .map(f -> f.substring(filename.lastIndexOf(".") + 1));

        // check file extension
        if (!ext.isPresent() || !ext.get().equals("polar")) {
            throw new Exceptions.PolarFileExtensionError(filename);
        }

        try {
            File file = new File(Paths.get(filename).toString());
            String hash = getFileChecksum(file);
            if (loadedNames.containsKey(filename)) {
                if (loadedNames.get(filename).equals(hash)) {
                    throw new Exceptions.PolarFileAlreadyLoadedError("File " + filename + " has already been loaded.");
                } else {
                    throw new Exceptions.PolarFileContentsChangedError(
                            "A file with the name " + filename + ", but different contents, has already been loaded.");
                }
            } else if (loadedContent.containsKey(hash)) {
                throw new Exceptions.PolarFileNameChangedError("A file with the same contents as " + filename
                        + " named " + loadedContent.get(hash) + "has already been loaded.");
            } else {
                loadStr(new String(Files.readAllBytes(Paths.get(filename))), filename);
                loadedNames.put(filename, hash);
                loadedContent.put(hash, filename);
            }
        } catch (NoSuchAlgorithmException e) {
            throw new PolarRuntimeException("Failed to hash file " + filename);
        } catch (FileNotFoundException e) {
            throw new Exceptions.PolarFileNotFoundError(filename);
        }

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
        return new Query(ffiPolar.newQueryFromStr(query), host.clone());
    }

    /**
     * Query for a predicate.
     */
    public Query query(Predicate query) throws Exceptions.OsoException {
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
     * Confirm that all queued inline queries succeed.
     */
    private void checkInlineQueries() throws Exceptions.OsoException, Exceptions.InlineQueryFailedError {
        Ffi.Query nextQuery = ffiPolar.nextInlineQuery();
        while (nextQuery != null) {
            if (!new Query(nextQuery, host).hasMoreElements()) {
                String source = nextQuery.source();
                throw new Exceptions.InlineQueryFailedError(source);
            }
            nextQuery = ffiPolar.nextInlineQuery();
        }
    }

    private static String getFileChecksum(File file) throws IOException, NoSuchAlgorithmException {
        // Get file input stream for reading the file content
        FileInputStream fis = new FileInputStream(file);

        // Use MD5 algorithm
        MessageDigest digest = MessageDigest.getInstance("MD5");

        // Create byte array to read data in chunks
        byte[] byteArray = new byte[1024];
        int bytesCount = 0;

        // Read file data and update in message digest
        while ((bytesCount = fis.read(byteArray)) != -1) {
            digest.update(byteArray, 0, bytesCount);
        }
        ;

        // close the stream; We don't need it now.
        fis.close();

        // Get the hash's bytes
        byte[] bytes = digest.digest();

        // This bytes[] has bytes in decimal format;
        // Convert it to hexadecimal format
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < bytes.length; i++) {
            sb.append(Integer.toString((bytes[i] & 0xff) + 0x100, 16).substring(1));
        }

        // return complete hash
        return sb.toString();
    }
}
