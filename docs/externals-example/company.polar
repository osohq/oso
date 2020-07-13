# policy-start
# Only members have read access to company-two
allow(actor, "read", company: Company) if
    company.name() = "company-two" and
    actor = company.members();

# Only executives have read access to company-one
allow(actor, "read", company: Company) if
    company.name() = "company-one" and
    actor = company.department_members("executive");
# policy-end

# Investors have read access to startups
allow(actor, "read", company: StartUp) if
    actor = company.investors();
