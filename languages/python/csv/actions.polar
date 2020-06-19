# Action groups
R("read");
W("write");
C("create");
U("unlink");

RW(action) := R(action) | W(action);
RWC(action) := RW(action) | C(action);
RWCU(action) := RWC(action) | U(action);


# Lookup role for user
role(user, role) := user.groups.id = role;

