package com.osohq.oso;

import com.osohq.oso.Exceptions.OsoException;
import com.osohq.oso.Exceptions.ParseError;
import com.osohq.oso.Exceptions.PolarRuntimeException;
import java.io.BufferedReader;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.ArrayList;
import org.json.JSONArray;

public class Polar {
  private Ffi.Polar ffiPolar;
  protected Host host; // visible for tests only
  private boolean polarRolesEnabled;

  public Polar() throws Exceptions.OsoException {
    ffiPolar = Ffi.get().polarNew();
    host = new Host(ffiPolar);
    polarRolesEnabled = false;

    // Register global constants.
    registerConstant(null, "nil");

    // Register built-in classes.
    registerClass(Boolean.class, "Boolean");
    registerClass(Integer.class, "Integer");
    registerClass(Double.class, "Float");
    registerClass(List.class, "List");
    registerClass(Map.class, "Dictionary");
    registerClass(String.class, "String");
    registerClass(RolesHelper.class, "__oso_internal_roles_helpers__");
  }

  /**
   * Clear the rules from the KB, but maintain all registered classes and calls.
   *
   * @throws Exceptions.OsoException
   */
  public void clearRules() throws Exceptions.OsoException {
    ffiPolar.clearRules();
    reinitializeRoles();
  }

  /**
   * Enqueue a polar policy file to be loaded. File contents are loaded into a String and saved
   * here, so changes to the file made after calls to loadFile will not be recognized. If the
   * filename already exists in the load queue, replace it.
   *
   * @throws Exceptions.PolarFileExtensionError On incorrect file extension.
   * @throws IOException If unable to open or read the file.
   */
  public void loadFile(String filename) throws IOException, OsoException {
    Optional<String> ext =
        Optional.ofNullable(filename)
            .filter(f -> f.contains("."))
            .map(f -> f.substring(filename.lastIndexOf(".") + 1));

    // check file extension
    if (!ext.isPresent() || !ext.get().equals("polar")) {
      throw new Exceptions.PolarFileExtensionError(filename);
    }

    try {
      loadStr(new String(Files.readAllBytes(Paths.get(filename))), filename);
    } catch (FileNotFoundException e) {
      throw new Exceptions.PolarFileNotFoundError(filename);
    }
  }

  /**
   * Load a Polar string into the KB (with filename).
   *
   * @param str Polar string to be loaded.
   * @param filename Name of the source file.
   */
  public void loadStr(String str, String filename) throws Exceptions.OsoException {
    ffiPolar.load(str, filename);
    checkInlineQueries();
    reinitializeRoles();
  }

  /**
   * Load a Polar string into the KB (without filename).
   *
   * @param str Polar string to be loaded.
   */
  public void loadStr(String str) throws Exceptions.OsoException {
    ffiPolar.load(str, null);
    checkInlineQueries();
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

  /** Start the Polar REPL. */
  public void repl() throws Exceptions.OsoException, IOException {
    repl(new String[0]);
  }

  /** Load the given files and start the Polar REPL. */
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

  public void reinitializeRoles() {
    if (!polarRolesEnabled) return;
    polarRolesEnabled = false;
    enableRoles();
  }

  public void enableRoles() throws Exceptions.OsoException {
    if (polarRolesEnabled) return;
    ffiPolar.enableRoles();

    List<List<HashMap<String, Object>>> allResults = new ArrayList<List<HashMap<String, Object>>>();

    Ffi.Query nextQuery = ffiPolar.nextInlineQuery();
    for (; nextQuery != null; nextQuery = ffiPolar.nextInlineQuery()) {
      Host dupHost = host.clone();
      dupHost.acceptExpression = true;
      Query query = new Query(nextQuery, dupHost, Map.of());
      if (!query.hasMoreElements()) {
        throw new Exceptions.InlineQueryFailedError(nextQuery.source());
      } else {
        allResults.add(query.results());
      }
    }

    for (List<HashMap<String, Object>> results : allResults) {
      for (HashMap<String, Object> result : results) {
        HashMap<String, Object> bs = new HashMap<String, Object>();
        result.forEach((k, v) -> {
          bs.put(k, host.toPolarTerm(v));
        });
        result.put("bindings", bs);
      }
    }

    ffiPolar.validateRolesConfig(new JSONArray(allResults).toString());
    this.polarRolesEnabled = true;
  }
}
