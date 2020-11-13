package com.osohq.oso.parity;

import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ArgumentsSource;

public class ParityTest {
  @ParameterizedTest
  @ArgumentsSource(TestCase.class)
  public void runTests(TestCase testCase) throws Exception {
    testCase.run();
  }
}
