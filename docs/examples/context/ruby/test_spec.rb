require "oso"
require_relative "02-context.rb"

RSpec.describe do
  def load_file(example_name)
      file = File.join(File.dirname(__FILE__), '..', example_name)
      OSO.load_file(file)
      OSO
  end

  context "01-context" do
    before do
      @oso = load_file("01-context.polar")
    end

    it "works" do
      ENV["ENV"] = "production"
      expect(@oso.allowed?(
        actor: "steve",
        action: "test",
        "resource": "policy")).to be false

      ENV["ENV"] = "development"
      expect(@oso.allowed?(
        actor: "steve",
        action: "test",
        "resource": "policy")).to be true
    end
  end
end
