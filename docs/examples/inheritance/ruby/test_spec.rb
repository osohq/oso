require "oso"

require_relative "inheritance_external.rb"

FILES = [
  "01-polar.polar",
  "02-nested-rule.polar",
  "03-specializer.polar",
  "04-one-specializer.polar"
]

RSpec.describe do
  before do
    @patient = "Bob"
    @med_staff = Actor.new(role: "medical_staff", treated: [@patient])
    @med_staff_bad_patient = Actor.new(role: "medical_staff", treated: ["Not Bob"])

    @reg_staff = Actor.new(role: "reg_staff", treated: [@patient])

    @order = Order.new(patient: @patient)
    @lab = Lab.new(patient: @patient)
    @test = Test.new(patient: @patient)
  end

  def load_file(example_name)
      oso = Oso::Oso.new

      file = File.join(File.dirname(__FILE__), '..', example_name)
      oso.load_file(file)
      oso
  end

  FILES.each do |file|
    context "#{file}" do
      before do
        @oso = load_file(file)
        setup(@oso)
      end

      it "parses" do
        @oso.load_queued_files
      end

      it "allows medical staff" do
        expect(@oso.allow(
          actor: @med_staff,
          action: "read",
          resource: @order
        )).to be true

        expect(@oso.allow(
          actor: @med_staff,
          action: "read",
          resource: @lab
        )).to be true

        expect(@oso.allow(
          actor: @med_staff,
          action: "read",
          resource: @test
        )).to be true
      end

      it "denies for mismatched patient" do
        expect(@oso.allow(
          actor: @med_staff_bad_patient,
          action: "read",
          resource: @order
        )).to be false

        expect(@oso.allow(
          actor: @med_staff_bad_patient,
          action: "read",
          resource: @lab
        )).to be false

        expect(@oso.allow(
          actor: @med_staff_bad_patient,
          action: "read",
          resource: @test
        )).to be false
      end

      it "denies for regular staff" do
          expect(@oso.allow(
            actor: @reg_staff,
            action: "read",
            resource: @order
          )).to be false

          expect(@oso.allow(
            actor: @reg_staff,
            action: "read",
            resource: @lab
          )).to be false

          expect(@oso.allow(
            actor: @reg_staff,
            action: "read",
            resource: @test
          )).to be false
      end
    end
  end
end
