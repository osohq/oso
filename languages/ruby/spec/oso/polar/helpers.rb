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

  def count(coll)
    coll.reduce(Hash.new(0)) { |c, x| c.tap { c[x] += 1 } }
  end

  def unord_eq(left, right)
    count(left) == count(right)
  end
end
