package com.osohq.oso;

import java.util.HashMap;
import java.util.List;

public class TypeConstraint extends Expression {
  public TypeConstraint(Object left, String typeName) {
    super(
        Operator.And,
        List.of(new Expression(Operator.Isa, List.of(left, new Pattern(typeName, new HashMap())))));
  }
}
