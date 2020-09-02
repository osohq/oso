require "oso"
require_relative "02-user_classes.rb"

RSpec.describe do
  def load_file(example_name)
      file = File.join(File.dirname(__FILE__), '..', example_name)
      OSO.load_file(file)
      OSO
  end

  context "user classes" do
    it "parses" do
        load_file("user_policy.polar")
    end
  end
end
