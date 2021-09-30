# frozen_string_literal: true

require_relative './helpers'
require_relative './parity'

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  context '#authorized_resources' do # rubocop:disable Metrics/BlockLength
    it 'handles classes with explicit names' do
      subject.register_class(
        Widget,
        name: 'Doohickey',
        fields: { id: Integer }
      )

      subject.load_str 'allow("gwen", "get", it: Doohickey) if it.id = 8;'
      check_authz 'gwen', 'get', Widget, [Widget.all[8]]
    end

    it 'handles queries that return known results' do
      subject.register_class(Widget, fields: { id: Integer })
      subject.register_constant(Widget.all.first, name: 'Prototype')
      subject.load_str <<~POL
        allow("gwen", "tag", w: Widget) if w = Prototype;
        allow("gwen", "tag", w: Widget) if w.id in [1,2,3];
      POL
      results = subject.authorized_resources 'gwen', 'tag', Widget

      expect(results).to contain_exactly(*Widget.all[0..3])
    end

    context 'when filtering data' do # rubocop:disable Metrics/BlockLength
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

      context 'with specializers in the rule head' do # rubocop:disable Metrics/BlockLength
        it 'works' do # rubocop:disable Metrics/BlockLength
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
              Log.all.each do |log|
                results = subject.authorized_resources log.foo, a + b, Log
                expect(results).to contain_exactly(log)
              end
            end
          end
        end
      end

      context 'for collection membership' do # rubocop:disable Metrics/BlockLength
        it 'can check if a value is in a field' do
          policy = 'allow("gwen", "get", foo: Foo) if 1 in foo.numbers and 2 in foo.numbers;'
          subject.load_str(policy)
          results = subject.authorized_resources('gwen', 'get', Foo)
          expected = Foo.all.select { |f| f.numbers.include?(1) and f.numbers.include?(2) }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can check if a field is in a value' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.numbers in [[1]];'
          subject.load_str(policy)
          results = subject.authorized_resources('gwen', 'get', Foo)
          expected = Foo.all.select { |f| f.numbers == [1] }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can check if a value is in a field on a direct relation' do
          policy = 'allow("gwen", "get", log: Log) if 1 in log.foo.numbers;'
          subject.load_str policy
          results = subject.authorized_resources('gwen', 'get', Log)
          expected = Log.all.select { |l| l.foo.numbers.include? 1 }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can check if a value is in a field on an indirect relation' do
          subject.load_str <<~POL
            allow("gwen", "get", log: Log) if
              foo in log.foo.bar.foos and
              0 in foo.numbers;
          POL
          results = subject.authorized_resources('gwen', 'get', Log)
          expected = Log.all.select { |l| l.foo.bar.foos.any? { |f| f.numbers.include? 0 } }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end
      end

      context 'for equality' do # rubocop:disable Metrics/BlockLength
        it 'can compare a field with a known value' do
          policy = 'allow("gwen", "get", foo: Foo) if foo.is_fooey = true;'
          subject.load_str(policy)
          results = subject.authorized_resources('gwen', 'get', Foo)
          expected = Foo.all.select(&:is_fooey)
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on the same object' do
          subject.load_str <<~POL
            allow("gwen", "put", bar: Bar) if
              bar.is_cool = bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Bar)
          expected = Bar.all.select { |b| b.is_cool == b.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on a related object' do
          subject.load_str <<~POL
            allow("gwen", "put", foo: Foo) if
              foo.bar.is_cool = foo.bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Foo)
          expected = Foo.all.select { |foo| foo.bar.is_cool == foo.bar.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on an indirectly related object' do
          subject.load_str <<~POL
            allow("gwen", "put", log: Log) if
              log.data = "world" and
              log.foo.bar.is_cool = log.foo.bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Log)
          expected = Log.all.select do |log|
            log.data == 'world' and log.foo.bar.is_still_cool == log.foo.bar.is_cool
          end
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'returns empty results for an impossible query' do
          subject.load_str <<~POL
            allow("gwen", "gwt", foo: Foo) if
              foo.is_fooey = true and
              foo.is_fooey = false;
          POL

          results = subject.authorized_resources('gwen', 'get', Foo)
          expect(results).to be_empty
        end

        it 'correctly applies constraints from other rules' do
          subject.load_str <<~POL
            f(bar: Bar) if bar.is_cool = true;
            g(bar: Bar) if bar.is_still_cool = true;
            h(bar: Bar) if foo in bar.foos and log in foo.logs and i(log);
            i(log: Log) if log.data = "world";
            allow("gwen", "get", bar: Bar) if
              f(bar) and g(bar) and h(bar);
          POL

          results = subject.authorized_resources('gwen', 'get', Bar)
          expected = Bar.all.find { |bar| bar.id == 'hello' }
          expect(results).to contain_exactly(expected)
        end
      end

      context 'for inequality' do # rubocop:disable Metrics/BlockLength
        it 'can compare two fields on the same object' do
          subject.load_str <<~POL
            allow("gwen", "get", bar: Bar) if
              bar.is_cool != bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'get', Bar)
          expected = Bar.all.reject { |b| b.is_cool == b.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on a related object' do
          subject.load_str <<~POL
            allow("gwen", "put", foo: Foo) if foo.bar.is_cool != foo.bar.is_still_cool;
          POL

          results = subject.authorized_resources('gwen', 'put', Foo)
          expected = Foo.all.reject { |foo| foo.bar.is_cool == foo.bar.is_still_cool }
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end

        it 'can compare two fields on an indirectly related object' do
          policy = <<~POL
            allow("gwen", "put", log: Log) if
              log.data = "goodbye" and
              log.foo.bar.is_cool != log.foo.bar.is_still_cool;
          POL
          subject.load_str(policy)

          results = subject.authorized_resources('gwen', 'put', Log)
          expected = Log.all.select do |log|
            log.data == 'goodbye' and log.foo.bar.is_still_cool != log.foo.bar.is_cool
          end
          expect(expected).not_to be_empty
          expect(results).to contain_exactly(*expected)
        end
      end

      it 'handles one-to-one relationships' do
        policy = <<~POL
          allow("gwen", "get", foo: Foo) if
            foo.is_fooey = true and
            foo.bar.is_cool = true;
        POL
        subject.load_str(policy)

        results = subject.authorized_resources('gwen', 'get', Foo)
        expected = Foo.all.select { |foo| foo.bar.is_cool and foo.is_fooey }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'handles one-to-many relationships' do
        policy = 'allow("gwen", "get", foo: Foo) if log in foo.logs and log.data = "goodbye";'
        subject.load_str policy
        expected = Foo.all.select { |foo| foo.id == 'fourth' }
        check_authz 'gwen', 'get', Foo, expected
      end

      it 'handles nested one-to-one relationships' do
        policy = <<~POL
          allow("gwen", "put", log: Log) if
            log.data = "goodbye" and
            log.foo.is_fooey = true and
            log.foo.bar.is_cool != true;
        POL
        subject.load_str(policy)

        results = subject.authorized_resources('gwen', 'put', Log)
        expected = Log.all.select { |log| log.data == 'goodbye' and log.foo.is_fooey and !log.foo.bar.is_cool }
        expect(expected).not_to be_empty
        expect(results).to contain_exactly(*expected)
      end

      it 'handles all the relationships at once' do
        policy = <<~POL
          allow(log: Log, "a", foo: Foo) if log in foo.logs;
          allow(log: Log, "b", foo: Foo) if foo = log.foo;
          allow(log: Log, "c", foo: Foo) if log.foo = foo and log in foo.logs;
          allow(log: Log, "d", foo: Foo) if log in foo.logs and log.foo = foo;
        POL
        subject.load_str policy
        log = Log.all.find { |l| l.foo_id == 'fourth' }
        foos = Foo.all.select { |foo| foo.id == 'fourth' }
        %w[a b c d].each { |x| check_authz log, x, Foo, foos }
      end
    end
  end
end
