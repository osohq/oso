require "oso"
require_relative "02-context.rb"

RSpec.describe do
  def load_file(example_name)
      oso = Oso::Oso.new

      file = File.join(File.dirname(__FILE__), '..', example_name)
      oso.load_file(file)
      oso
  end

  context "01-context" do
    before do
      @oso = load_file("01-context.polar")
      setup(@oso)
    end

    it "works" do
      ENV["ENV"] = "production"
      expect(@oso.allow(
        actor: "steve",
        action: "test",
        "resource": "policy")).to be false

      ENV["ENV"] = "development"
      expect(@oso.allow(
        actor: "steve",
        action: "test",
        "resource": "policy")).to be true
    end
  end
end
