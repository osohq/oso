package com.osohq.oso;

import static java.nio.charset.StandardCharsets.UTF_8;
import static java.util.Optional.ofNullable;

import com.osohq.oso.Exceptions.OsoException;
import com.osohq.oso.Exceptions.ParseError;
import com.osohq.oso.Exceptions.PolarRuntimeException;
import java.io.BufferedReader;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.json.JSONArray;

public class Polar {

  private static final String POLAR_EXTENTION = "polar";

  private Ffi.Polar ffiPolar;
  protected Host host; // visible for tests only

  public Polar() throws Exceptions.OsoException {
    ffiPolar = Ffi.get().polarNew();
    host = new Host(ffiPolar);

    // Register global constants.
    registerConstant(null, "nil");

    // Register built-in classes.
    registerClass(Boolean.class, "Boolean");
    registerClass(Integer.class, "Integer");
    registerClass(Double.class, "Float");
    registerClass(List.class, "List");
    registerClass(Map.class, "Dictionary");
    registerClass(String.class, "String");
  }

  /**
   * Clear the rules from the KB, but maintain all registered classes and calls.
   *
   * @throws Exceptions.OsoException
   */
  public void clearRules() throws Exceptions.OsoException {
    ffiPolar.clearRules();
  }

  /**
   * Load Polar policy files. File contents are loaded into a String and saved here, so changes to a
   * file made after a call to loadFiles will not be recognized.
   *
   * @throws Exceptions.PolarFileExtensionError On incorrect file extension.
   * @throws Exceptions.PolarFileNotFoundError On nonexistent file.
   * @throws Exceptions.InlineQueryFailedError On a failed inline query.
   * @throws IOException If unable to open or read the file.
   */
  public void loadFiles(String[] filenames) throws IOException, OsoException {
    if (filenames == null || filenames.length == 0) {
      return;
    }

    JSONArray sources = new JSONArray();

    for (String filename : filenames) {
      // check file extension
      ofNullable(filename)
          .filter(f -> f.contains("."))
          .map(f -> f.substring(filename.lastIndexOf(".") + 1))
          .filter(extention -> extention.equals(POLAR_EXTENTION))
          .orElseThrow(() -> new Exceptions.PolarFileExtensionError(filename));

      try {
        String contents = new String(Files.readAllBytes(Paths.get(filename)));
        Source source = new Source(contents, filename);
        sources.put(source.toJSON());
      } catch (FileNotFoundException e) {
        throw new Exceptions.PolarFileNotFoundError(filename);
      }
    }

    loadSources(sources);
  }

  /**
   * Load Polar policy files from resources. File contents are loaded into a String and saved here,
   * so changes to a file made after a call to loadFiles will not be recognized.
   *
   * @throws Exceptions.PolarFileExtensionError On incorrect file extension.
   * @throws Exceptions.PolarFileNotFoundError On nonexistent file.
   * @throws Exceptions.InlineQueryFailedError On a failed inline query.
   * @throws IOException If unable to open or read the file.
   */
  public void loadFilesFromResources(String... filenames) throws IOException, OsoException {
    if (filenames == null || filenames.length == 0) {
      return;
    }

    JSONArray sources = new JSONArray();
    for (String filename : filenames) {
      // check file extension
      ofNullable(filename)
          .filter(f -> f.contains("."))
          .map(f -> f.substring(filename.lastIndexOf(".") + 1))
          .filter(extention -> extention.equals(POLAR_EXTENTION))
          .orElseThrow(() -> new Exceptions.PolarFileExtensionError(filename));

      try (InputStream inputStream = getClass().getResourceAsStream(filename)) {
        if (inputStream == null) {
          throw new Exceptions.PolarFileNotFoundError(filename);
        }

        String contents = new String(inputStream.readAllBytes(), UTF_8);

        sources.put(new Source(contents, filename).toJSON());
      }
    }

    loadSources(sources);
  }

  /**
   * Load a Polar policy file. File contents are loaded into a String and saved here, so changes to
   * the file made after a call to loadFile will not be recognized.
   *
   * @throws Exceptions.PolarFileExtensionError On incorrect file extension.
   * @throws Exceptions.PolarFileNotFoundError On nonexistent file.
   * @throws Exceptions.InlineQueryFailedError On a failed inline query.
   * @throws IOException If unable to open or read the file.
   * @deprecated {@link #loadFile(String)} has been deprecated in favor of {@link
   *     #loadFiles(String[])} as of the 0.20 release. Please see changelog for migration
   *     instructions: https://docs.osohq.com/project/changelogs/2021-09-15.html
   */
  @Deprecated
  public void loadFile(String filename) throws IOException, OsoException {
    System.err.println(
        "`Oso.loadFile` has been deprecated in favor of `Oso.loadFiles` as of the 0.20"
            + " release.\n\n"
            + "Please see changelog for migration instructions:"
            + " https://docs.osohq.com/project/changelogs/2021-09-15.html");
    loadFiles(new String[] {filename});
  }

  /**
   * Load a Polar string into the KB (with filename).
   *
   * @param str Polar string to be loaded.
   * @param filename Name of the source file.
   * @throws Exceptions.InlineQueryFailedError On a failed inline query.
   */
  public void loadStr(String str, String filename) throws OsoException {
    JSONArray sources = new JSONArray();
    Source source = new Source(str, filename);
    sources.put(source.toJSON());
    loadSources(sources);
  }

  /**
   * Load a Polar string into the KB (without filename).
   *
   * @param str Polar string to be loaded.
   * @throws Exceptions.InlineQueryFailedError On a failed inline query.
   */
  public void loadStr(String str) throws OsoException {
    loadStr(str, null);
  }

  /** Query for a predicate, parsing it first. */
  public Query query(String query) throws OsoException {
    return query(query, Map.of(), false);
  }

  /** Query for a predicate, parsing it first and optionally accepting an expression. */
  public Query query(String query, boolean acceptExpression) throws OsoException {
    return query(query, Map.of(), acceptExpression);
  }

  /** Query for a predicate, parsing it first and applying bindings */
  public Query query(String query, Map<String, Object> bindings) throws Exceptions.OsoException {
    return query(query, bindings, false);
  }

  /**
   * Query for a predicate, parsing it first, applying bindings and optionally accepting an
   * expression.
   */
  public Query query(String query, Map<String, Object> bindings, boolean acceptExpression)
      throws Exceptions.OsoException {
    Host new_host = host.clone();
    new_host.setAcceptExpression(acceptExpression);
    return new Query(ffiPolar.newQueryFromStr(query), new_host, bindings);
  }

  /** Query for a predicate. */
  public Query query(Predicate query) throws Exceptions.OsoException {
    Host new_host = host.clone();
    String pred = new_host.toPolarTerm(query).toString();
    return new Query(ffiPolar.newQueryFromTerm(pred), new_host, Map.of());
  }

  /** Query for a predicate, optionally accepting expressions in the result. */
  public Query query(Predicate query, boolean acceptExpression) throws Exceptions.OsoException {
    return query(query, Map.of(), acceptExpression);
  }

  /**
   * Query for a predicate, applying bindings and optionally accepting the expression type as a
   * result.
   *
   * @param acceptExpression Set to true to accept an Expression as a result from the VM.
   */
  public Query query(Predicate query, Map<String, Object> bindings, boolean acceptExpression)
      throws Exceptions.OsoException {
    Host new_host = host.clone();
    new_host.setAcceptExpression(acceptExpression);
    String pred = new_host.toPolarTerm(query).toString();
    return new Query(ffiPolar.newQueryFromTerm(pred), new_host, bindings);
  }

  /**
   * Query for a rule.
   *
   * @param rule Rule name, e.g. "f" for rule "f(x)".
   * @param args Variable list of rule arguments.
   */
  public Query queryRule(String rule, Object... args) throws OsoException {
    return queryRule(rule, Map.of(), args);
  }

  /**
   * Query for a rule.
   *
   * @param rule Rule name, e.g. "f" for rule "f(x)".
   * @param args Variable list of rule arguments.
   */
  public Query queryRule(String rule, Map<String, Object> bindings, Object... args)
      throws Exceptions.OsoException {
    Host new_host = host.clone();
    String pred = new_host.toPolarTerm(new Predicate(rule, Arrays.asList(args))).toString();
    return new Query(ffiPolar.newQueryFromTerm(pred), new_host, bindings);
  }

  /**
   * Query for a rule, and check if it has any results. Returns true if there are results, and false
   * if not.
   *
   * @param rule Rule name, e.g. "f" for rule "f(x)".
   * @param args Variable list of rule arguments.
   */
  public boolean queryRuleOnce(String rule, Object... args) throws OsoException {
    return queryRule(rule, Map.of(), args).hasMoreElements();
  }

  /** Start the Polar REPL. */
  public void repl() throws Exceptions.OsoException, IOException {
    repl(new String[0]);
  }

  /** Load the given files and start the Polar REPL. */
  public void repl(String[] files) throws Exceptions.OsoException, IOException {
    loadFiles(files);

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
        query = new Query(ffiQuery, host, Map.of());
      } catch (PolarRuntimeException e) {
        System.out.println(e.toString());
        continue;
      }

      if (!query.hasMoreElements()) {
        System.out.println("false");
      } else {
        do {
          HashMap<String, Object> result = query.nextElement();
          if (result.size() == 0) {
            System.out.println("true");
          } else {
            result.forEach(
                (variable, value) -> System.out.println(variable + " = " + value.toString()));
          }
        } while (query.hasMoreElements());
      }
    }
  }

  public static void main(String[] args) throws Exceptions.OsoException, IOException {
    new Polar().repl(args);
  }

  /** Register a Java class with Polar. */
  public void registerClass(Class<?> cls)
      throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
    registerClass(cls, cls.getName());
  }

  /** Register a Java class with Polar using an alias. */
  public void registerClass(Class<?> cls, String name)
      throws Exceptions.DuplicateClassAliasError, Exceptions.OsoException {
    host.cacheClass(cls, name);
    registerConstant(cls, name);
  }

  /** Registers `value` as a Polar constant variable called `name`. */
  public void registerConstant(Object value, String name) throws Exceptions.OsoException {
    ffiPolar.registerConstant(host.toPolarTerm(value).toString(), name);
  }

  /** Register MROs, load Polar code, and check inline queries. */
  private void loadSources(JSONArray sources) throws OsoException {
    host.registerMros();
    ffiPolar.load(sources);
    checkInlineQueries();
  }

  /** Confirm that all queued inline queries succeed. */
  private void checkInlineQueries()
      throws Exceptions.OsoException, Exceptions.InlineQueryFailedError {
    Ffi.Query nextQuery = ffiPolar.nextInlineQuery();
    while (nextQuery != null) {
      if (!new Query(nextQuery, host, Map.of()).hasMoreElements()) {
        String source = nextQuery.source();
        throw new Exceptions.InlineQueryFailedError(source);
      }
      nextQuery = ffiPolar.nextInlineQuery();
    }
  }
}
