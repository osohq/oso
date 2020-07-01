# user-start
OSO ||= Oso.new

class User
  ...
end

OSO.register_class(User)
# user-end

# expense-start
class Expense
  ...
end

OSO.register_class(Expense)
# expense-end