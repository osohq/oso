package com.osohq.oso;

import java.util.Objects;

public class Variable {
  private String name;

  public Variable(String name) {
    this.name = name;
  }

  @Override
  public String toString() {
    return name;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) return true;
    if (o == null || getClass() != o.getClass()) return false;
    Variable variable = (Variable) o;
    return name.equals(variable.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name);
  }
}
