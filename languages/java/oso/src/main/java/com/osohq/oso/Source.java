package com.osohq.oso;

import org.json.JSONObject;

public class Source {
  private String src;
  private String filename; // nullable

  public Source(String src, String filename) {
    this.src = src;
    this.filename = filename;
  }

  public JSONObject toJSON() {
    JSONObject source = new JSONObject();
    source.put("src", src);
    source.put("filename", filename);
    return source;
  }
}
