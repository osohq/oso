# frozen_string_literal: true

require_relative './helpers'
require_relative './parity'

bars = Bar.all
foos = Foo.all
logs = Log.all

hello_bar = bars.find { |b| b.id == 'hello' }
goodbye_bar = bars.find { |b| b.id == 'goodbye' }
# hershey_bar = bars.find { |b| b.id == 'hershey' }

something_foo = foos.find { |f| f.id == 'something' }
another_foo = foos.find { |f| f.id == 'another' }
# third_foo = foos.find { |f| f.id == 'third' }
fourth_foo = foos.find { |f| f.id == 'fourth' }

fourth_log_a = logs.find { |l| l.id == 'a' }
third_log_b = logs.find { |l| l.id == 'b' }
# another_log_c = logs.find { |l| l.id == 'c' }

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  context 'data filtering parity' do # rubocop:disable Metrics/BlockLength
    it 'test_model' do
      subject.load_str <<~POL
        allow(_, _, _: Foo{id: "something"});
      POL
      check_authz 'gwen', 'get', Foo, [something_foo]

      subject.clear_rules
      subject.load_str <<~POL
        allow(_, _, _: Foo{id: "something"});
        allow(_, _, _: Foo{id: "another"});
      POL
      check_authz 'gwen', 'get', Foo, [another_foo, something_foo]
    end

    it 'test_authorize_scalar_attribute_eq' do
      subject.load_str <<~POL
        allow(_: Bar, "read", _: Foo{is_fooey:true});
        allow(bar: Bar, "read", _: Foo{bar:bar});
      POL
      bars.each do |bar|
        expected = foos.select { |f| f.is_fooey || f.bar == bar }
        check_authz bar, 'read', Foo, expected
      end
    end

    it 'test_authorize_scalar_attribute_condition' do
      subject.load_str <<~POL
        allow(bar: Bar{is_cool:true}, "read", _: Foo{bar:bar});
        allow(_: Bar, "read", _: Foo{bar:b, is_fooey:true}) if b.is_cool;
        allow(_: Bar{is_still_cool: true}, "read", foo: Foo) if
          foo.bar.is_cool = false;
      POL
      bars.each do |bar|
        expected = foos.select do |foo|
          foo.bar.is_cool && foo.bar == bar ||
            foo.bar.is_cool && foo.is_fooey ||
            !foo.bar.is_cool && bar.is_still_cool
        end
        check_authz bar, 'read', Foo, expected
      end
    end

    it 'test_in_multiple_attribute_relationship' do
      subject.load_str <<~POL
        allow(_, "read", _: Foo{is_fooey: false});
        allow(bar, "read", _: Foo{bar: bar});
        allow(_, "read", foo: Foo) if
          1 in foo.numbers and
          foo.bar.is_cool;
        allow(_, "read", foo: Foo) if
          2 in foo.numbers and
          foo.bar.is_cool;
      POL

      bars.each do |bar|
        expected = foos.select do |foo|
          !foo.is_fooey ||
            bar == foo.bar ||
            (foo.bar.is_cool && [1, 2].any? { |n| foo.numbers.include? n })
        end
        check_authz bar, 'read', Foo, expected
      end
    end

    it 'test_nested_relationship_many_single' do
      subject.load_str <<~POL
        allow(log: Log, "read", bar: Bar) if log.foo in bar.foos;
      POL
      logs.each do |log|
        expected = bars.select { |bar| bar.foos.include? log.foo }
        check_authz log, 'read', Bar, expected
      end
    end

    it 'test_nested_relationship_many_many' do
      subject.load_str <<~POL
        allow(log: Log, "read", bar: Bar) if
          foo in bar.foos and log in foo.logs;
      POL
      logs.each do |log|
        expected = bars.select do |bar|
          bar.foos.any? { |foo| foo.logs.include? log }
        end
        check_authz log, 'read', Bar, expected
      end
    end

    it 'test_nested_relationship_many_many_constrained' do
      subject.load_str <<~POL
        allow(log: Log{data: "steve"}, "read", bar: Bar) if
          foo in bar.foos and
          log in foo.logs;
      POL

      logs.each do |log|
        expected = bars.select do |bar|
          log.data == 'steve' && bar.foos.any? { |foo| foo.logs.include? log }
        end

        if log.data == 'steve'
          expect(expected).not_to be_empty
        else
          expect(expected).to be_empty
        end

        check_authz log, 'read', Bar, expected
      end
    end

    it 'test_partial_in_collection' do
      subject.load_str 'allow(bar, "read", foo: Foo) if foo in bar.foos;'
      bars.each do |bar|
        check_authz bar, 'read', Foo, bar.foos
      end
    end

    it 'test_empty_constraints_in' do
      subject.load_str 'allow(_, "read", foo: Foo) if _ in foo.logs;'
      expected = foos.select { |foo| foo.logs.any? }
      check_authz 'gwen', 'read', Foo, expected
    end

    it 'test_in_with_constraints_but_no_matching_object' do
      subject.load_str <<~POL
        allow(_, "read", foo: Foo) if log in foo.logs and log.data = "nope";
      POL

      check_authz 'gwen', 'read', Foo, []
    end

    it 'test_redundant_in_on_same_field' do
      subject.load_str <<~POL
        allow(_, _, foo: Foo) if
          m in foo.numbers and
          n in foo.numbers and
          m = 1 and n = 2;
      POL

      expected = foos.select { |foo| [1, 2].all? { |n| foo.numbers.include? n } }
      expect(expected).to contain_exactly(fourth_foo)
      check_authz 'gwen', 'get', Foo, expected
    end

    it 'test_unify_ins' do
      subject.load_str <<~POL
        allow(_, _, _: Bar{foos:foos}) if
          foo in foos and
          goo in foos and
          foo = goo;
      POL

      expected = bars.select { |bar| bar.foos.any? }
      expect(expected).to contain_exactly(hello_bar, goodbye_bar)
      check_authz 'gwen', 'read', Bar, expected
    end

    it 'test_partial_isa_with_path' do
      subject.load_str <<~POL
        allow(_, _, foo: Foo) if check(foo.bar);
        check(bar: Bar) if bar.id = "goodbye";   # this should match
        check(foo: Foo) if foo.bar.id = "hello"; # this shouldn't match
      POL
      check_authz 'gwen', 'read', Foo, goodbye_bar.foos
    end

    it 'test_no_relationships' do
      subject.load_str <<~POL
        allow(_, _, foo: Foo) if foo.is_fooey;
      POL
      expected = foos.select(&:is_fooey)
      check_authz 'gwen', 'get', Foo, expected
    end

    it 'test_neq' do
      subject.load_str <<~POL
        allow(_, action, foo: Foo) if foo.bar.id != action;
      POL

      bars.each do |bar|
        expected = foos.reject { |foo| foo.bar == bar }
        check_authz 'gwen', bar.id, Foo, expected
      end
    end

    it 'test_relationship' do
      subject.load_str <<~POL
        allow("steve", "get", foo: Foo) if
          foo.bar = bar and
          bar.is_cool and
          foo.is_fooey;
      POL
      expected = foos.select { |foo| foo.bar.is_cool and foo.is_fooey }
      check_authz 'steve', 'get', Foo, expected
    end

    it 'test_duplex_relationship' do
      subject.load_str <<~POL
        allow(_, _, foo: Foo) if foo in foo.bar.foos;
      POL
      check_authz 'gwen', 'get', Foo, foos
    end

    it 'test_scalar_in_list' do
      subject.load_str <<~POL
        allow(_, _, _: Foo{bar:bar}) if bar.is_cool in [true, false];
      POL
      check_authz 'gwen', 'get', Foo, foos
    end

    it 'test_var_in_vars' do
      subject.load_str <<~POL
        allow(_, _, foo: Foo) if
          log in foo.logs and
          log.data = "hello";
      POL
      expected = foos.select do |foo|
        foo.logs.any? { |log| log.data == 'hello' }
      end
      check_authz 'gwen', 'get', Foo, expected
    end

    it 'test_parent_child_cases' do
      subject.load_str <<~POL
        allow(_: Log{foo: foo},   0, foo: Foo);
        allow(log: Log,           1, _: Foo{logs: logs}) if log in logs;
        allow(log: Log{foo: foo}, 2, foo: Foo{logs: logs}) if log in logs;
      POL
      0.upto(2) do |n|
        logs.each do |log|
          check_authz log, n, Foo, [log.foo]
        end
      end
    end

    it 'test_specializers' do
      subject.load_str <<~POL
        allow(foo: Foo,             "NoneNone", log) if foo = log.foo;
        allow(foo,                  "NoneCls",  log: Log) if foo = log.foo;
        allow(foo,                  "NoneDict", _: {foo:foo});
        allow(foo,                  "NonePtn",  _: Log{foo: foo});
        allow(foo: Foo,             "ClsNone",  log) if log in foo.logs;
        allow(foo: Foo,             "ClsCls",   log: Log) if foo = log.foo;
        allow(foo: Foo,             "ClsDict",  _: {foo: foo});
        allow(foo: Foo,             "ClsPtn",   _: Log{foo: foo});
        allow(_: {logs: logs},      "DictNone", log) if log in logs;
        allow(_: {logs: logs},      "DictCls",  log: Log) if log in logs;
        allow(foo: {logs: logs},    "DictDict", log: {foo: foo}) if log in logs;
        allow(foo: {logs: logs},    "DictPtn",  log: Log{foo: foo}) if log in logs;
        allow(_: Foo{logs: logs},   "PtnNone",  log) if log in logs;
        allow(_: Foo{logs: logs},   "PtnCls",   log: Log) if log in logs;
        allow(foo: Foo{logs: logs}, "PtnDict",  log: {foo: foo}) if log in logs;
        allow(foo: Foo{logs: logs}, "PtnPtn",   log: Log{foo: foo}) if log in logs;
      POL
      parts = %w[None Cls Dict Ptn]
      parts.each do |a|
        parts.each do |b|
          logs.each { |log| check_authz log.foo, a + b, Log, [log] }
        end
      end
    end

    it 'test_ground_object_in_collection' do
      subject.load_str 'allow(_, _, _: Foo{numbers:ns}) if 1 in ns and 2 in ns;'
      check_authz 'gwen', 'get', Foo, [fourth_foo]
    end

    it 'test_var_in_value' do
      subject.load_str 'allow(_, _, log: Log) if log.data in ["goodbye", "world"];'
      check_authz 'steve', 'get', Log, [third_log_b, fourth_log_a]
    end

    it 'test_field_eq' do
      subject.load_str 'allow(_, _, _: Bar{is_cool: cool, is_still_cool: cool});'
      expected = bars.select { |bar| bar.is_cool == bar.is_still_cool }
      check_authz 'gwen', 'get', Bar, expected
    end

    it 'test_field_neq' do
      subject.load_str 'allow(_, _, bar: Bar) if bar.is_cool != bar.is_still_cool;'
      expected = bars.reject { |bar| bar.is_cool == bar.is_still_cool }
      check_authz 'gwen', 'get', Bar, expected
    end

    it 'test_const_in_coll' do
      magic = 1
      subject.register_constant magic, name: 'magic'
      subject.load_str 'allow(_, _, foo: Foo) if magic in foo.numbers;'
      expected = foos.select { |foo| foo.numbers.include? magic }
      check_authz 'gwen', 'get', Foo, expected
    end

    it 'test_param_field' do
      subject.load_str 'allow(data, id, _: Log{data: data, id: id});'
      logs.each { |log| check_authz log.data, log.id, Log, [log] }
    end

    # not supported
    xit 'test_or' do
      subject.load_str <<~POL
        allow(_, _, r: Foo) if not (r.id = "something" and r.bar_id = "hello");
      POL
      results = subject.authorized_resources 'steve', 'get', Foo
      expect(results).to have_length 2
    end

    it 'test_field_cmp_rel_field' do
      subject.load_str 'allow(_, _, foo: Foo) if foo.bar.is_cool = foo.is_fooey;'
      expected = foos.select { |foo| foo.bar.is_cool == foo.is_fooey }
      check_authz 'gwen', 'get', Foo, expected
    end

    it 'test_field_cmp_rel_rel_field' do
      subject.load_str('allow(_, _, log: Log) if log.data = log.foo.bar.id;')
      expected = [fourth_log_a]
      check_authz 'gwen', 'get', Log, expected
    end

    # not supported
    xit 'test_const_not_in_coll' do
      magic = 1
      subject.register_constant magic, name: 'magic'
      subject.load_str 'allow(_, _, foo: Foo) if not (magic in foo.numbers);'
      expected = foos.reject { |foo| foo.numbers.include? magic }
      check_authz 'gwen', 'get', Foo, expected
    end

    # not supported
    xit 'test_forall_in_collection' do
      subject.load_str 'allow(_, _, bar: Bar) if forall(foo in bar.foos, foo.is_fooey = true);'
      expected = bars.select { |bar| bar.foos.all?(&:is_fooey) }
      check_authz 'gwen', 'get', Bar, expected
    end

    # buggy
    xit 'test_unify_ins_neq' do
      subject.load_str <<~POL
        allow(_, _, _: Bar{foos:foos}) if
          foo in foos and
          goo in foos and
          foo != goo;
      POL

      expected = bars.select { |bar| bar.foos.length >= 2 }
      check_authz 'gwen', 'read', Bar, expected
    end

    it 'test_unify_ins_field_eq' do
      subject.load_str <<~POL
        allow(_, _, _: Bar{foos:foos}) if
            foo in foos and
            goo in foos and
            foo.id = goo.id;
      POL
      expected = bars.select { |bar| bar.foos.any? }
      check_authz 'gwen', 'get', Bar, expected
    end

    # buggy
    xit 'test_deeply_nested_in' do
      subject.load_str <<~POL
        allow(_, _, a: Foo) if
            b in a.bar.foos and b != a and
            c in b.bar.foos and c != b and
            d in c.bar.foos and d != c and
            e in d.bar.foos and e != d;
      POL
      expected = bars.select { |bar| bar.foos.length >= 2 }
      check_authz 'gwen', 'get', Foo, expected
    end

    it 'test_in_intersection' do
      subject.load_str <<~POL
        allow(_, _, foo: Foo) if
          num in foo.numbers and
          goo in foo.bar.foos and
          goo != foo and
          num in goo.numbers;
      POL

      expect(subject.authorized_resources('gwen', 'get', Foo)).to be_empty
    end
  end

  before do # rubocop:disable Metrics/BlockLength
    subject.register_class(
      Bar,
      fields: {
        id: String,
        is_cool: PolarBoolean,
        is_still_cool: PolarBoolean,
        foos: Relation.new(
          kind: 'many',
          other_type: 'Foo',
          my_field: 'id',
          other_field: 'bar_id'
        )
      }
    )

    subject.register_class(
      Log,
      fields: {
        id: String,
        foo_id: String,
        data: String,
        foo: Relation.new(
          kind: 'one',
          other_type: 'Foo',
          my_field: 'foo_id',
          other_field: 'id'
        )
      }
    )

    subject.register_class(
      Foo,
      fields: {
        id: String,
        bar_id: String,
        is_fooey: PolarBoolean,
        numbers: Array,
        bar: Relation.new(
          kind: 'one',
          other_type: 'Bar',
          my_field: 'bar_id',
          other_field: 'id'
        ),
        logs: Relation.new(
          kind: 'many',
          other_type: 'Log',
          my_field: 'id',
          other_field: 'foo_id'
        )
      }
    )
  end
end
