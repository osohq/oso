package com.osohq.oso;

import java.util.HashMap;
import java.util.Objects;

public class Pattern {
  private String tag; // nullable
  private HashMap<String, Object> fields;

  public Pattern(String tag, HashMap<String, Object> fields) {
    this.tag = tag;
    this.fields = fields;
  }

  public String getTag() {
    return tag;
  }

  public void setTag(String tag) {
    this.tag = tag;
  }

  public HashMap<String, Object> getFields() {
    return fields;
  }

  public void setFields(HashMap<String, Object> fields) {
    this.fields = fields;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o) return true;
    if (o == null || getClass() != o.getClass()) return false;
    Pattern pattern = (Pattern) o;
    return Objects.equals(tag, pattern.tag) && fields.equals(pattern.fields);
  }

  @Override
  public int hashCode() {
    return Objects.hash(tag, fields);
  }
}
