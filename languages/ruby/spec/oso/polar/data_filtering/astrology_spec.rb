# frozen_string_literal: true

require_relative './helpers'
require 'sqlite3'
require 'active_record'
DB_FILE = 'astro_test.db'

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  context 'an astrological matchmaking app' do # rubocop:disable Metrics/BlockLength
    it 'applies sound elemental reasoning' do
      subject.load_str <<~POL
        allow("the water of aquarius", "slake", x: Person) if
          x.sign.element in ["air", "earth", "water"];
        allow("the venom of scorpio", "intoxicate", x: Person) if
          x.sign.element in ["air", "fire"];
        allow("the venom of scorpio", "intoxicate", x: Person) if
          x.sign.ruler in ["saturn", "neptune"];
      POL

      water_winners = Person.joins(:sign).where.not(signs: { element: 'fire' })
      check_authz 'the water of aquarius', 'slake', Person, water_winners

      venom_victims =
        Person.joins(:sign).where(signs: { element: %w[air fire] })
              .or(Person.joins(:sign).where(signs: { ruler: %w[saturn neptune] }))
      check_authz 'the venom of scorpio', 'intoxicate', Person, venom_victims
    end

    it 'assigns auspicious matches' do
      # FIXME(gw) probably not astrologically correct
      subject.load_str <<~POL
        align(_: Sign{ruler: r},   _: Sign{ruler: r});
        align(_: Sign{element: e}, _: Sign{element: e});
        allow(a: Person, "match", b: Person) if
          a != b and align(a.sign, b.sign);
      POL

      compatible_signs = lambda do |sign|
        Sign.where(element: sign.element).or Sign.where(ruler: sign.ruler)
      end

      compatible_people = lambda do |person|
        Person.where.not(name: person.name).where(sign: compatible_signs[person.sign])
      end

      Person.all.each do |person|
        check_authz person, 'match', Person, compatible_people[person]
      end
    end
  end

  before do # rubocop:disable Metrics/BlockLength
    File.delete DB_FILE if File.exist? DB_FILE
    db = SQLite3::Database.new DB_FILE
    db.execute <<-SQL
      create table signs (
        name varchar(16) not null primary key,
        element varchar(8) not null,
        ruler varchar(8) not null
      );
    SQL

    db.execute <<-SQL
      create table people (
        name varchar(32) not null primary key,
        sign_name varchar(16) not null
      );
    SQL

    ActiveRecord::Base.establish_connection(
      adapter: 'sqlite3',
      database: DB_FILE
    )

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
     %w[pisces water jupiter]].each do |name, element, ruler|
      Sign.create(name: name, element: element, ruler: ruler)
    end

    [%w[robin scorpio],
     %w[pat taurus],
     %w[dylan virgo],
     %w[terry libra],
     %w[chris aquarius],
     %w[tyler leo],
     %w[eden cancer],
     %w[dakota capricorn],
     %w[charlie aries],
     %w[alex gemini],
     %w[sam pisces],
     %w[avery sagittarius]].each do |name, sign|
      Person.create(name: name, sign_name: sign)
    end

    subject.register_class(
      Sign,
      fields: {
        name: String,
        element: String,
        ruler: String,
        people: Relation.new(
          kind: 'many',
          other_type: 'Person',
          my_field: 'name',
          other_field: 'sign_name'
        )
      }
    )

    subject.register_class(
      Person,
      fields: {
        name: String,
        sign_name: String,
        sign: Relation.new(
          kind: 'one',
          other_type: 'Sign',
          my_field: 'sign_name',
          other_field: 'name'
        )
      }
    )
  end
end

class Sign < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = 'name'
  has_many :people, foreign_key: :sign_name
end

class Person < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = 'name'
  belongs_to :sign, foreign_key: :sign_name
end
