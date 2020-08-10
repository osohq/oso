require "oso"

RSpec.describe do
  before do
    stub_const('User', Class.new do
      attr_accessor :name, :role
      def initialize(name: nil, role: nil)
        @name = name
        @role = role
      end
    end)
  end

  def load_file(example_name)
      oso = Oso::Oso.new
      oso.register_class(User)

      file = File.join(File.dirname(__FILE__), '..', example_name)
      oso.load_file(file)
      oso
  end

  context "01-simple" do
    it "parses" do
      load_file("01-simple.polar")
    end
  end

  context "02-simple" do
    it "parses" do
      load_file("02-simple.polar")
    end
  end

  context "05-external.polar" do
    before do
      @oso = load_file("05-external.polar")
    end

    it "works" do
      expect(@oso.allowed?(
        actor: User.new(role: "employee"),
        action: "submit",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(role: "admin"),
        action: "approve",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(role: "employee"),
        action: "approve",
        resource: "expense")).to be false
      expect(@oso.allowed?(
        actor: User.new(role: "accountant"),
        action: "view",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(name: "greta"),
        action: "approve",
        resource: "expense")).to be true
    end
  end

  context "06-external.polar" do
    before do
      @oso = load_file("06-external.polar")
    end

    it "works" do
      expect(@oso.allowed?(
        actor: User.new(role: "employee"),
        action: "submit",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(role: "employee"),
        action: "view",
        resource: "expense")).to be false
      expect(@oso.allowed?(
        actor: User.new(role: "employee"),
        action: "approve",
        resource: "expense")).to be false

      expect(@oso.allowed?(
        actor: User.new(role: "accountant"),
        action: "submit",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(role: "accountant"),
        action: "view",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(role: "accountant"),
        action: "approve",
        resource: "expense")).to be false

      expect(@oso.allowed?(
        actor: User.new(role: "admin"),
        action: "submit",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(role: "admin"),
        action: "view",
        resource: "expense")).to be true
      expect(@oso.allowed?(
        actor: User.new(role: "admin"),
        action: "approve",
        resource: "expense")).to be true
    end
  end
end
