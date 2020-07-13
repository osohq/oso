person("sam", "scott");
person("david", "hatch");

company("oso");

employee(company("oso"), person("sam", "scott"));
employee(company("oso"), person("david", "hatch"));

?= person("sam", "scott");

?= person(first, last) and first = "sam" and last = "scott";
?= person(first, last) and first = "david" and last = "hatch";

?= employee(company("oso"), employee) and employee = person("david", "hatch");

osoEmployee(employee) if employee(company("oso"), "employee");
