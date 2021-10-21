# frozen_string_literal: true

require_relative './helpers'
require 'sqlite3'
require 'active_record'

D = Oso::Polar::Data
RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  context 'new filters' do
    Join = D::ArelJoin
    Src = D::ArelSource
    Select = D::ArelSelect
    Field = D::Proj
    Value = D::Value
    persons = Src[Person]
    signs = Src[Sign]
    person_name = Field[persons, :name]
    person_sign_name = Field[persons, :sign_name]
    sign_name = Field[signs, :name]
    context 'astrology' do
      it 'field value no join' do
        # person.name = 'eden'
        select = Select[
          persons,
          person_name,
          Value['eden']
        ]
        expect(select.to_query.to_a).to eq [eden]

        # person.name != 'eden'
        result = Select[
          persons,
          person_name,
          Value['eden'],
          kind: :neq
        ].to_a
        expect(result.length).to be 11
        expect(result).not_to include eden
      end

      it 'field value one join' do
        # person.sign.name = 'cancer'
        result = Select[
          Join[persons, person_sign_name, sign_name, signs],
          sign_name,
          Value['cancer']
        ].to_a
        expect(result).to eq [eden]
        # person.sign.name != 'cancer'
        result = Select[
          Join[persons, person_sign_name, sign_name, signs],
          sign_name,
          Value['cancer'],
          kind: :neq
        ].to_a
        expect(result.length).to be 11
        expect(result).not_to include eden
      end

      it 'field field one join' do
        # person.name = person.sign.name
        result = Select[ # * from
          Join[persons, person_sign_name, sign_name, signs],
          person_name,
          sign_name,
        ].to_a
        expect(result).to eq [leo]

        # person.name != person.sign.name
        result = Select[ # * from
          Join[persons, person_sign_name, sign_name, signs],
          person_name,
          sign_name,
          kind: :neq
        ].to_a
        expect(result.length).to be 11
        expect(result).not_to include leo
      end

      DB_FILE = 'astro_test.db'
      before do # rubocop:disable Metrics/BlockLength
        File.delete DB_FILE if File.exist? DB_FILE
        db = SQLite3::Database.new DB_FILE
        db.execute <<~SQL
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
      end

      let(:eden) { Person.find 'eden' }
      let(:leo) { Person.find 'leo' }
    end
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

class User < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
  has_many :org_roles, foreign_key: :user_name
  has_many :repo_roles, foreign_key: :user_name
end

class Repo < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :org, foreign_key: :org_name
  has_many :issues, foreign_key: :repo_name
  has_many :repo_roles, foreign_key: :repo_name
end

class Org < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  has_many :users, foreign_key: :org_name
  has_many :repos, foreign_key: :org_name
  has_many :org_roles, foreign_key: :org_name
end

class Issue < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  self.primary_key = :name
  belongs_to :repo, foreign_key: :repo_name
end

class RepoRole < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  belongs_to :user, foreign_key: :user_name
  belongs_to :repo, foreign_key: :repo_name
end

class OrgRole < ActiveRecord::Base
  include DFH::ActiveRecordFetcher
  belongs_to :user, foreign_key: :user_name
  belongs_to :org, foreign_key: :org_name
end
