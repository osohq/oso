# frozen_string_literal: true

module Helpers
  def next_result(results)
    results.next.transform_values(&:to_ruby)
  end

  def qvar(polar, query, var)
    next_result(polar.query_str(query))[var]
  end
end
