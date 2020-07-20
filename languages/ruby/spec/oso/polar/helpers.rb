# frozen_string_literal: true

module Helpers
  def query(polar, query)
    polar.send(:query, query).to_a
  end

  def qvar(polar, query, var, one: false)
    results = query(polar, query)
    if one
      results.first[var]
    else
      results.map { |r| r[var] }
    end
  end
end
