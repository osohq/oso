### Role definitions

role(actor: User, "employee") if
    actor.role = "employee" or
    role(actor, "accountant");

role(actor: User, "accountant") if
    actor.role = "accountant" or
    role(actor, "admin");

role(actor: User, "admin") if
    actor.role = "admin";

?= role(new User("alice"), "employee");
?= role(new User("ebrahim"), "employee");
?= role(new User("ebrahim"), "accountant");
