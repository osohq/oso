# frozen_string_literal: true

require_relative './helpers'

Wizard = DFH.record(:name, :books, :spell_levels) do
  def spells
    Spell.all.select do |spell|
      books.include?(spell.school) and spell_levels.include?(spell.level)
    end
  end
end

Familiar = DFH.record :name, :kind, :wizard_name
Spell = DFH.record :name, :school, :level
Spell.new('teleport other',    'thaumaturgy', 7)
Spell.new('wish',              'thaumaturgy', 9)
Spell.new('cure light wounds', 'necromancy',  1)
Spell.new('identify',          'divination',  1)
Spell.new('call familiar',     'summoning',   1)
Spell.new('call ent',          'summoning',   7)
Spell.new('magic missile',     'destruction', 1)
Spell.new('liquify organ',     'destruction', 5)
Spell.new('call dragon',       'summoning',   9)
Spell.new('know alignment',    'divination',  6)

RSpec.describe Oso::Oso do # rubocop:disable Metrics/BlockLength
  let(:level) { ->(n) { 1.upto(n).to_a } }
  let(:policy_file) { File.join(__dir__, 'magic.polar') }
  let(:gandalf) { Wizard.new('gandalf', %w[divination destruction], level[4]) }
  let(:galadriel) { Wizard.new('galadriel', %w[thaumaturgy divination inscription], level[7]) }
  let(:baba_yaga) { Wizard.new('baba yaga', %w[necromancy summoning destruction], level[8]) }
  let(:shadowfax) { Familiar.new('shadowfax', 'horse', 'gandalf') }
  let(:brown_jenkin) { Familiar.new('brown jenkin', 'rat', 'baba yaga') }
  let(:gimli) { Familiar.new('gimli', 'dwarf', 'galadriel') }
  let(:hedwig) { Familiar.new('hedwig', 'owl', 'galadriel') }

  before do # rubocop:disable Metrics/BlockLength
    subject.register_class(
      Wizard,
      fields: {
        name: String,
        books: Array,
        spell_levels: Array,
        familiars: Relation.new(
          kind: 'many',
          other_type: 'Familiar',
          my_field: 'name',
          other_field: 'wizard_name'
        )
      }
    )

    subject.register_class(
      Spell,
      fields: {
        name: String,
        school: String,
        level: Integer
      }
    )

    subject.register_class(
      Familiar,
      fields: {
        name: String,
        kind: String,
        wizard_name: String,
        wizard: Relation.new(
          kind: 'one',
          other_type: 'Wizard',
          my_field: 'wizard_name',
          other_field: 'name'
        )
      }
    )

    subject.load_files [policy_file]
  end

  context 'wizards' do
    it 'can cast any spell in their spellbook up to their level' do
      Wizard.all.each do |wiz|
        check_authz wiz, 'cast', Spell, wiz.spells
      end
    end

    it 'can ride their horse familiars' do
      check_authz gandalf, 'ride', Familiar, [shadowfax]
      check_authz galadriel, 'ride', Familiar, []
      check_authz baba_yaga, 'ride', Familiar, []
    end

    it 'can groom their familiars' do
      check_authz baba_yaga, 'groom', Familiar, [brown_jenkin]
      check_authz galadriel, 'groom', Familiar, [hedwig, gimli]
      check_authz gandalf, 'groom', Familiar, [shadowfax]
    end

    context 'having mastered inscription' do
      it 'can inscribe any spell they can cast' do
        check_authz galadriel, 'inscribe', Spell, galadriel.spells
        check_authz gandalf, 'inscribe', Spell, []
        check_authz baba_yaga, 'inscribe', Spell, []
      end
    end
  end

  context 'rat familiars' do
    it 'can groom other familiars, except owls (predator)' do
      check_authz brown_jenkin, 'groom', Familiar, [gimli, brown_jenkin, shadowfax]
    end
    it 'can groom their wizard' do
      check_authz brown_jenkin, 'groom', Wizard, [baba_yaga]
    end
  end
end
