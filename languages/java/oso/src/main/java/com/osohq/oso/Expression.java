package com.osohq.oso;

import java.util.List;
import java.util.Objects;

public class Expression {
  private Operator operator;
  private List<Object> args;

  public Expression(Operator operator, List<Object> args) {
    this.operator = operator;
    this.args = args;
  }

  public Operator getOperator() {
    return operator;
  }

  public void setOperator(Operator operator) {
    this.operator = operator;
  }

  public List<Object> getArgs() {
    return args;
  }

  public void setArgs(List<Object> args) {
    this.args = args;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) return true;
    if (o == null || getClass() != o.getClass()) return false;
    Expression that = (Expression) o;
    return operator == that.operator && args.equals(that.args);
  }

  @Override
  public int hashCode() {
    return Objects.hash(operator, args);
  }
}
