# frozen_string_literal: true

module Helpers
  def qvar(polar, query, var)
    results = polar.query_str(query)
    results.next[var]
  end
end
