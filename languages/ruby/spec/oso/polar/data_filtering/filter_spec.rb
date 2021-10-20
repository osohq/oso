# frozen_string_literal: true

require_relative './helpers'
require 'sqlite3'
require 'active_record'

D = Oso::Polar::Data
RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  DB_FILE = 'astro_test.db'

  context 'new filter representation' do
    Join = D::ArelJoin
    Model = D::ArelSource
    Select = D::ArelSelect
    Field = D::ArelProj
    Value = D::Value

    let(:eden) { Person.find 'eden' }
    let(:leo) { Person.find 'leo' }

    it 'can select a record from a table based on a field/value equality' do
      select = Select[
        Model[Person],
        Field[Person, :name],
        Value['eden']
      ]
      expect(select.to_query.to_a).to eq [eden]
    end

    it 'can select a record from a table based on field/value inequality' do
      result = Select[
        Model[Person],
        Field[Person, :name],
        Value['eden'],
        kind: :neq
      ].to_a
      expect(result.length).to be 11
      expect(result).not_to include eden
    end

    it 'can select a record based on field/value equality in a joined table' do
      result = Select[
        Join[Field[Person, :sign_name],
             Field[Sign, :name]],
        Field[Sign, :name],
        Value['cancer']
      ].to_a
      expect(result).to eq [eden]
    end

    it 'can select a record based on field/value inequality in a joined table' do
      result = Select[
        Join[Field[Person, :sign_name],
             Field[Sign, :name]],
        Field[Sign, :name],
        Value['cancer'],
        kind: :neq
      ].to_a
      expect(result.length).to be 11
      expect(result).not_to include eden
    end

    it 'can select a record based on field/field equality in two different tables' do
      result = Select[
        Join[Field[Person, :sign_name],
             Field[Sign, :name]],
        Field[Person, :name],
        Field[Sign, :name]
      ].to_a
      expect(result).to eq [leo]
    end

    it 'can select a record based on field/field inequality in two different tables' do
      result = Select[
        Join[Field[Person, :sign_name],
             Field[Sign, :name]],
        Field[Person, :name],
        Field[Sign, :name],
        kind: :neq
      ].to_a
      expect(result.length).to be 11
      expect(result).not_to include leo
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
     %w[leo leo],
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
