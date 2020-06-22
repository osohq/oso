### Role definitions

role(actor: User, "employee") :=
    actor.role = "employee"
    | role(actor, "accountant");

role(actor: User, "accountant") :=
    actor.role = "accountant"
    | role(actor, "admin");

role(actor: User, "admin") :=
    actor.role = "admin";

?= role(new User{name: "alice"}, "employee");
?= role(new User{name: "ebrahim"}, "employee");
?= role(new User{name: "ebrahim"}, "accountant");
