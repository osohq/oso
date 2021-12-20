# docs: begin-adapter
# Example data filtering adapter for ActiveRecord
class ActiveRecordAdapter < ::Oso::Polar::Data::Adapter
  def build_query(filter) # rubocop:disable Metrics/AbcSize, Metrics/MethodLength
    types = filter.types
    query = filter.relations.reduce(filter.model.all) do |q, rel|
      rec = types[rel.left].fields[rel.name]
      q.joins(
        "INNER JOIN #{rel.right.table_name} ON " \
        "#{rel.left.table_name}.#{rec.my_field} = " \
        "#{rel.right.table_name}.#{rec.other_field}"
      )
    end

    filter.conditions.map do |conjs|
      conjs.reduce(query) do |q, conj|
        q.where(*sqlize(conj))
      end
    end.reduce(:or).distinct
  end

  def execute_query(query)
    query.to_a
  end

  OPS = {
    'Eq' => '=', 'In' => 'IN', 'Nin' => 'NOT IN', 'Neq' => '!='
  }.freeze

  private

  def sqlize(cond)
    args = []
    lhs = add_side cond.left, args
    rhs = add_side cond.right, args
    args.unshift "#{lhs} #{OPS[cond.cmp]} #{rhs}"
  end

  def add_side(side, args)
    if side.is_a? ::Oso::Polar::Data::Filter::Projection
      "#{side.source.table_name}.#{side.field || side.source.primary_key}"
    else
      args.push side
      '?'
    end
  end
end
# docs: end-adapter
