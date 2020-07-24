OSO ||= Oso.new

class User
    attr_reader :name
    attr_reader :is_admin

    def initialize(name, is_admin)
        @name = name
        @is_admin = is_admin
    end
end

user = User.new("alice", true)
raise "should be allowed" unless OSO.allow(user, "foo", "bar")