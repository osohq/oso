person("sam", "scott");
person("david", "hatch");

company("oso");

employee(company("oso"), person("sam", "scott"));
employee(company("oso"), person("david", "hatch"));

?= person("sam", "scott");

?= person(first, last), first = "sam", last = "scott";
?= person(first, last), first = "david", last = "hatch";

?= employee(company("oso"), employee), employee = person("david", "hatch");

osoEmployee(employee) if employee(company("oso"), "employee");
