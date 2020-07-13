package com.osohq.oso;

import java.util.*;

import junit.framework.Test;
import junit.framework.TestCase;
import junit.framework.TestSuite;

public class PolarTest extends TestCase {
    protected Polar p;

    /**
     * Create the test case
     *
     * @param testName name of the test case
     */
    public PolarTest(String testName) {
        super(testName);
    }

    @Override
    public void setUp() {
        try {
            p = new Polar();
        } catch (Exceptions.OsoException e) {
            throw new Error(e);
        }
    }

    /**
     * @return the suite of tests being tested
     */
    public static Test suite() {
        return new TestSuite(PolarTest.class);
    }

    /**
     * Rigourous Test :-)
     */
    public void testApp() {
        assertTrue(true);
    }

    public static void testLoadAndQueryStr() throws Exception {
        Polar p = new Polar();

        p.loadStr("f(1);");
        Query query = p.queryStr("f(x)");
        assertEquals(List.of(Map.of("x", 1)), query.results());
    }
}
