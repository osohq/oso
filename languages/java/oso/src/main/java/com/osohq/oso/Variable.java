package com.osohq.oso;

public class Variable {
  private String name;

  public Variable(String name) {
    this.name = name;
  }

  @Override
  public String toString() {
    return name;
  }
}
