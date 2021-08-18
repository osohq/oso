# frozen_string_literal: true

# Test helpers.
module Helpers
  def query(polar, query)
    polar.query(query).to_a
  end

  def qvar(polar, query, var, one: false)
    results = query(polar, query)
    if one
      results.first[var]
    else
      results.map { |r| r[var] }
    end
  end

  def count(a)
    a.reduce(Hash.new 0) {|c, x| c[x] += 1; c }
  end

  def unord_eq(a, b)
    count(a) == count(b)
  end
end
