# frozen_string_literal: true

require_relative './helpers'
require 'sqlite3'
require 'active_record'
require 'oso/polar/data/adapter/active_record_adapter'

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  before do
    subject.data_filtering_adapter =
      ::Oso::Polar::Data::Adapter::ActiveRecordAdapter.new
  end

  context 'new filters' do # rubocop:disable Metrics/BlockLength
    class Sign < ActiveRecord::Base
      has_many :people
      belongs_to :planet
    end

    class Person < ActiveRecord::Base
      belongs_to :sign
    end

    class Planet < ActiveRecord::Base
      has_many :signs
    end

    context 'astrology' do # rubocop:disable Metrics/BlockLength
      context '#authorized_query parity' do # rubocop:disable Metrics/BlockLength
        before do # rubocop:disable Metrics/BlockLength
          subject.register_class(
            Person,
            fields: {
              name: String,
              sign_id: Integer,
              sign: Relation.new(
                kind: 'one',
                other_type: 'Sign',
                my_field: 'sign_id',
                other_field: 'id'
              )
            }
          )
          subject.register_class(
            Planet,
            fields: {
              name: String,
              signs: Relation.new(
                kind: 'many',
                other_type: 'Sign',
                my_field: 'id',
                other_field: 'planet_id'
              )
            }
          )
          subject.register_class(
            Sign,
            fields: {
              name: String,
              element: String,
              planet: Relation.new(
                kind: 'one',
                other_type: 'Planet',
                my_field: 'planet_id',
                other_field: 'id'
              ),
              people: Relation.new(
                kind: 'many',
                other_type: 'Person',
                my_field: 'id',
                other_field: 'sign_id'
              )
            }
          )
        end
        it 'test_authorize_scalar_attribute_eq' do
          subject.load_str <<~POL
            allow(_: Person, "read", _: Sign{element: "fire"});
            allow(_: Person{sign}, "read", sign);
          POL
          query = subject.authorized_query(sam, 'read', Sign)
          expected_signs = %w[pisces aries sagittarius leo].map { |n| Sign.find_by_name n }
          expect(query.to_a).to contain_exactly(*expected_signs)
        end

        it 'test_authorize_scalar_attribute_condition' do
          subject.load_str <<~POL
            # signs ruled by jupiter can read their own people
            # this rule relies on direct object comparison (aka `field == None`) working correctly :)
            allow(sign: Sign{planet}, "read", _: Person{sign}) if planet.name = "jupiter";
            # every sign can read a pisces named sam
            allow(_: Sign, "read", _: Person {sign, name: "sam"}) if sign.name = "pisces";
            # earth signs can read people with air signs
            allow(_: Sign{element: "earth"}, "read", _: Person{sign}) if sign.element = "air";
          POL

          test = lambda do |person, sign|
            (person.sign == sign && sign.planet.name == 'jupiter') ||
              (person.name == 'sam' && person.sign.name == 'pisces') ||
              (sign.element == 'earth' && person.sign.element == 'air')
          end
          Sign.all.each do |sign|
            expected = Person.all.select do |person|
              test[person, sign]
            end
            query = subject.authorized_query sign, 'read', Person
            expect(query.to_a).to contain_exactly(*expected)
          end
        end

        it 'test_nested_relationship_many_single' do
          subject.load_str <<~POL
            allow(_: Person{sign}, _, _: Planet{signs}) if sign in signs;
          POL
          Person.all.each do |person|
            query = subject.authorized_query person, nil, Planet
            expect(query.to_a).to eq [person.sign.planet]
          end
        end

        it 'test_nested_relationship_many_many' do
          subject.load_str <<~POL
            allow(person: Person, _, _: Planet{signs}) if
              person in signs.people;
          POL

          Person.all.each do |person|
            query = subject.authorized_query person, nil, Planet
            expect(query.to_a).to eq [person.sign.planet]
          end
        end

        it 'test_nested_relationship_many_many_constrained' do
          subject.load_str <<~POL
            allow(person: Person{name: "eden"}, _, _: Planet{signs}) if
              sign in signs and person in sign.people;
          POL

          Person.all.each do |person|
            query = subject.authorized_query person, nil, Planet
            if person == eden
              expect(query.to_a).to eq [person.sign.planet]
            else
              expect(query.to_a).to eq []
            end
          end
        end

        it 'test_partial_in_collection' do
          subject.load_str(
            'allow(_: Planet{signs}, _, sign: Sign) if sign in signs;'
          )
          Planet.all.each do |planet|
            query = subject.authorized_query planet, nil, Sign
            expect(query.to_a).to contain_exactly(*planet.signs)
          end
        end

        it 'test_empty_constraints_in' do
          subject.load_str <<~POL
            allow(_, _, _: Planet{signs}) if _ in signs;
          POL
          query = subject.authorized_query 'gwen', 'get', Planet
          expect(query.to_a).to contain_exactly(*Planet.where.not(name: 'pluto'))
        end

        it 'test_in_with_constraints_but_no_matching_object' do
          subject.load_str <<~POL
            allow(_, _, sign: Sign) if
              p in sign.people and
              p.name = "graham";
          POL
          query = subject.authorized_query 'gwen', 'get', Sign
          expect(query.to_a).to be_empty
        end

        it 'test_redundant_in_on_same_field' do
          subject.load_str <<~POL
            allow(_, _, _: Sign{people}) if
              a in people and b in people and
              a != b;
          POL

          query = subject.authorized_query 'gwen', 'read', Sign
          expect(query.to_a).to be_empty
        end

        it 'test_unify_ins' do
          subject.load_str <<~POL
            allow(_, _, _: Planet{signs}) if
              s in signs and t in signs and
              s = t;
          POL
          query = subject.authorized_query 'gwen', 'read', Planet
          expect(query.to_a).to contain_exactly(*Planet.where.not(name: 'pluto'))
        end

        it 'test_partial_isa_with_path' do
          subject.load_str <<~POL
            allow(_, _, _: Person{sign}) if check(sign);
            check(_: Sign{ name: "cancer" });
            check(_: Person{sign}) if sign.name = "leo";
          POL
          query = subject.authorized_query 'gwen', 'read', Person
          expected = Person.all.select { |person| person.sign.name == 'cancer' }

          expect(query.to_a).to contain_exactly(*expected)
        end

        it 'test_no_relationships' do
          subject.load_str 'allow(_, _, _: Sign{element:"fire"});'
          query = subject.authorized_query 'gwen', 'read', Sign
          expect(query.to_a).to contain_exactly(*Sign.where(element: 'fire'))
        end

        it 'test_neq' do
          subject.load_str 'allow(_, action, _: Sign{name}) if name != action;'
          query = subject.authorized_query 'gwen', 'libra', Sign
          expect(query.to_a).to contain_exactly(*Sign.where.not(name: 'libra'))
        end

        it 'test_relationship' do
          subject.load_str <<~POL
            allow(_, _, _: Person{ name: "eden", sign }) if sign.name = "cancer";
          POL

          query = subject.authorized_query 'gwen', 'read', Person
          expect(query.to_a).to eq([eden])
        end

        xit 'test_duplex_relationship' do
          subject.load_str <<~POL
            allow(_, _, sign: Sign) if sign in sign.planet.signs;
          POL

          query = subject.authorized_query 'gwen', 'read', Sign
          expect(query.to_a).to contain_exactly(*Sign.all)
        end

        it 'test_scalar_in_list' do
          subject.load_str 'allow(_, _, _: Sign{planet}) if planet.name in ["sun", "moon"];'
          query = subject.authorized_query 'gwen', 'read', Sign
          expected = Sign.all.select { |sign| %w[sun moon].include? sign.planet.name }
          expect(query.to_a).to contain_exactly(*expected)
        end

        it 'test_var_in_vars' do
          subject.load_str <<~POL
            allow(_, _, _: Sign{people}) if
              person in people and
              person.name = "eden";
          POL
          query = subject.authorized_query('gwen', 'read', Sign)
          expect(query.to_a).to eq [eden.sign]
        end

        it 'test_parent_child_cases' do
          subject.load_str <<~POL
            allow(_: Person{sign}, 0, sign: Sign);
            allow(person: Person, 1, _: Sign{people}) if person in people;
            allow(person: Person{sign}, 2, sign: Sign{people}) if person in people;
          POL

          0.upto(2) do |n|
            Person.all.each do |person|
              query = subject.authorized_query(person, n, Sign)
              expect(query.to_a).to eq [person.sign]
            end
          end
        end

        it 'test_specializers' do # rubocop:disable Metrics/BlockLength
          subject.load_str <<~POL
            allow(sign, "NoneNone", person) if person.sign = sign;
            allow(sign, "NoneCls", person: Person) if person.sign = sign;
            allow(sign, "NoneDict", _: {sign});
            allow(sign, "NonePtn", _: Person{sign});
            allow(sign: Sign, "ClsNone", person) if sign = person.sign;
            allow(sign: Sign, "ClsCls", person: Person) if sign = person.sign;
            allow(sign: Sign, "ClsDict", _: {sign});
            allow(sign: Sign, "ClsPtn", _: Person{sign});
            allow(_: {people}, "DictNone", person) if person in people;
            allow(_: {people}, "DictCls", person: Person) if person in people;
            allow(sign: {people}, "DictDict", person: {sign}) if person in people;
            allow(sign: {people}, "DictPtn", person: Person{sign}) if person in people;
            allow(_: Sign{people}, "PtnNone", person) if person in people;
            allow(_: Sign{people}, "PtnCls", person: Person) if person in people;
            allow(sign: Sign{people}, "PtnDict", person: {sign}) if person in people;
            allow(sign: Sign{people}, "PtnPtn", person: Person{sign}) if person in people;
          POL
          parts = %w[None Cls Dict Ptn]
          parts.each do |a|
            parts.each do |b|
              Person.all.each do |person|
                nom = a + b
                query = subject.authorized_query person.sign, nom, Person
                expect(query.to_a.unshift(nom)).to eq [nom, person]
              end
            end
          end
        end

        it 'test_or' do
          subject.load_str <<~POL
            allow(_, _, _: Sign{name, element}) if name = "leo" or element = "air";
          POL
          results = subject.authorized_query 'steve', 'get', Sign
          expected = Sign.where(name: 'leo').or(Sign.where(element: 'air'))
          expect(results.to_a).to contain_exactly(*expected)
        end

        xit 'test_ground_object_in_collection' do
        end

        it 'test_inequality_operators' do
          %w[< > <= >=].each do |op|
            subject.clear_rules
            subject.load_str "allow(_: Person{id:p}, _, _: Sign{id:s}) if p #{op} s;"
            query = subject.authorized_query(leo, 'get', Sign)
            expected = Sign.all.select { |s| eval "leo.id #{op} s.id" } # rubocop:disable Security/Eval, Style/EvalWithLocation, Lint/UnusedBlockArgument
            expect(query.to_a).to contain_exactly(*expected)
          end
        end

        it 'test_forall' do
          subject.load_str <<~POL
            allow(_: Planet{signs}, _, _: Person) if
              forall(sign in signs, sign.element != "fire");
          POL

          result = subject.authorized_resources(saturn, 'get', Person)
          expect(result).to contain_exactly(*Person.all)

          result = subject.authorized_resources(the_sun, 'get', Person)
          expect(result).to be_empty
        end

        it 'test_forall_forall' do
          subject.load_str <<~POL
            allow(_: Planet{signs}, _, _) if
              forall(sign in signs,
                forall(person in sign.people,
                  person.name != "sam"));
          POL
          result = subject.authorized_resources(sam.sign.planet, 'get', Person)
          expect(result).to be_empty
          result = subject.authorized_resources(mars, 'get', Person)
          expect(result).to contain_exactly(*Person.all)
        end

        it 'test_forall_not_in_relation' do
          subject.load_str <<~POL
            # planet can _ person if planet is not person's sign's planet
            allow(_: Planet{signs}, _, person: Person) if
              forall(sign in signs, not person in sign.people);
          POL
          Planet.all.each do |planet|
            result = subject.authorized_resources(planet, 'get', Person)
            expected = Person.joins(:sign).where.not(sign: planet.signs)
            expect(result).to contain_exactly(*expected)
          end
        end

        it 'test_not_in_relation' do
          subject.load_str <<~POL
            allow(_: Sign{people}, _, person: Person) if not person in people;
          POL
          Sign.all.each do |sign|
            result = subject.authorized_resources(sign, 'get', Person)
            expected = Person.joins(:sign).where.not(sign: sign)
            expect(result).to contain_exactly(*expected)
          end
        end

        it 'test_var_in_value' do
          subject.load_str 'allow(_, _, _: Person{name}) if name in ["leo", "mercury"];'
          query = subject.authorized_query('gwen', 'get', Person)
          expect(query.to_a).to contain_exactly(leo, mercury)
        end

        it 'test_field_eq' do
          subject.load_str 'allow(_, _, _: Person{name, sign}) if name = sign.name;'
          query = subject.authorized_query 'gwen', 'read', Person
          expect(query.to_a).to eq([leo])
        end

        it 'test_field_neq' do
          subject.load_str 'allow(_, _, _: Person{name, sign}) if name != sign.name;'
          query = subject.authorized_query 'gwen', 'read', Person
          expect(query.to_a).to contain_exactly(*Person.where.not(name: 'leo'))
        end

        it 'test_param_field' do
          subject.load_str 'allow(planet, element, _: Sign{planet, element});'
          Sign.all.each do |sign|
            query = subject.authorized_query sign.planet, sign.element, Sign
            expect(query.to_a).to eq [sign]
          end
        end

        it 'test_field_cmp_rel_field' do
          subject.load_str 'allow(_, _, _: Person{name, sign}) if name = sign.name;'
          query = subject.authorized_query 'gwen', 'read', Person
          expect(query.to_a).to eq [leo]
        end

        it 'test_field_cmp_rel_rel_field' do
          subject.load_str <<~POL
            allow(_, _, _: Person{name, sign}) if name = sign.planet.name;
          POL
          query = subject.authorized_query 'gwen', 'read', Person
          expect(query.to_a).to eq [mercury]
        end

        it 'test_in_with_scalar' do
          subject.load_str <<~POL
            allow(_, _, planet: Planet) if
              s in planet.signs and
              s.name = "scorpio";
          POL
          query = subject.authorized_query 'gwen', 'read', Planet
          expect(query.to_a).to eq [mars]
        end
      end

      DB_FILE = 'astro_test.db'
      before do # rubocop:disable Metrics/BlockLength
        File.delete DB_FILE if File.exist? DB_FILE
        db = SQLite3::Database.new DB_FILE
        db.execute <<~SQL
          create table signs (
            id integer primary key autoincrement,
            name varchar(16) unique not null,
            element varchar(8) not null,
            planet_id int not null
          );
        SQL

        db.execute <<~SQL
          create table people (
            id integer primary key autoincrement,
            name varchar(32) unique not null,
            sign_id integer not null
          );
        SQL

        db.execute <<~SQL
          create table planets (
            id integer primary key autoincrement,
            name varchar(8) unique not null
          );
        SQL

        ActiveRecord::Base.establish_connection(
          adapter: 'sqlite3',
          database: DB_FILE
        )

        %w[mars venus mercury moon sun jupiter saturn pluto].each do |name|
          Planet.create name: name
        end

        [%w[aries fire mars],
         %w[taurus earth venus],
         %w[gemini air mercury],
         %w[cancer water moon],
         %w[leo fire sun],
         %w[virgo earth mercury],
         %w[libra air venus],
         %w[scorpio water mars],
         %w[sagittarius fire jupiter],
         %w[capricorn earth saturn],
         %w[aquarius air saturn],
         %w[pisces water jupiter]].each do |name, element, planet|
          Sign.create(name: name, element: element, planet: Planet.find_by_name(planet))
        end

        [%w[robin scorpio],
         %w[pat taurus],
         %w[mercury virgo],
         %w[terry libra],
         %w[chris aquarius],
         %w[leo leo],
         %w[eden cancer],
         %w[dakota capricorn],
         %w[charlie aries],
         %w[alex gemini],
         %w[sam pisces],
         %w[avery sagittarius]].each do |name, sign|
          Person.create(name: name, sign: Sign.find_by_name(sign))
        end
      end

      let(:eden) { Person.find_by_name 'eden' }
      let(:leo) { Person.find_by_name 'leo' }
      let(:mercury) { Person.find_by_name 'mercury' }
      let(:mars) { Planet.find_by_name 'mars' }
      let(:sam) { Person.find_by_name 'sam' }
      let(:saturn) { Planet.find_by_name 'saturn' }
      let(:the_sun) { Planet.find_by_name 'sun' }
    end
  end
end
