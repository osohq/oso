# policy-start
# Only members have read access to company-two
allow(actor, "read", company: Company) :=
    company.name() = "company-two",
    actor = company.members();

# Only executives have read access to company-one
allow(actor, "read", company: Company) :=
    company.name() = "company-one",
    actor = company.department_members("executive");
# policy-end

# Investors have read access to startups
allow(actor, "read", company: StartUp) :=
    actor = company.investors();
