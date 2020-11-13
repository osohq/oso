package com.osohq.oso.parity;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;

import com.fasterxml.jackson.dataformat.yaml.YAMLMapper;
import com.osohq.oso.Oso;
import com.osohq.oso.parity.classes.IterableClass;
import com.osohq.oso.parity.classes.UnitClass;

import org.junit.jupiter.api.extension.ExtensionContext;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.ArgumentsProvider;

public class TestCase implements ArgumentsProvider {
    public static class CaseUnit {
        public String description;
        public String query;
        public String err;
        public List<HashMap<String, Object>> result;
    }

    public String name;
    public String description;
    public List<String> policies;
    public List<CaseUnit> cases;

    Oso getOso() throws Exception {
        Oso oso = new Oso();

        oso.registerClass(IterableClass.class, "IterableClass");
        oso.registerClass(UnitClass.class, "UnitClass");

        for (String policy : policies) {
            oso.loadFile(policy);
        }

        return oso;
    }

    void run() throws Exception {
        Oso oso = getOso();
        for (CaseUnit caseUnit : cases) {
            try {
                assertEquals(oso.query(caseUnit.query).results(), caseUnit.result);
            } catch (Exception e) {
                if (caseUnit.err != "") {
                    System.out.println(String.format("Expected: %s\nGot: %s", caseUnit.err, e.toString()));
                    Pattern p = Pattern.compile(caseUnit.err);
                    Matcher m = p.matcher(e.toString());
                    assertTrue(m.find());
                } else {
                    throw e;
                }
            }
        }
    }

    protected Set<Path> listFilesUsingDirectoryStream(String dir) throws IOException {
        Set<Path> fileList = new HashSet<>();
        try (DirectoryStream<Path> stream = Files.newDirectoryStream(Paths.get(dir))) {
            for (Path path : stream) {
                System.out.println(String.format("File: %s", path));
                if (!Files.isDirectory(path)) {
                    fileList.add(path);
                }
            }
        }
        System.out.println(String.format("Files: %s", fileList));
        return fileList;
    }

    @Override
    public Stream<? extends Arguments> provideArguments(ExtensionContext context) throws Exception {
        System.out.println("Getting arguments");
        YAMLMapper mapper = new YAMLMapper();
        mapper.findAndRegisterModules();
        return listFilesUsingDirectoryStream("../../../test/spec/").stream().map(path -> {
            TestCase testCase = null;
            try {
                testCase = mapper.readValue(path.toFile(), TestCase.class);
                System.out.println(String.format("TestCase: %s", testCase));
            } catch (Exception e) {
                System.err.println(e.toString());
                e.printStackTrace();
            }
            return testCase;
        }).map(testCase -> Arguments.of(testCase));
    }

}
