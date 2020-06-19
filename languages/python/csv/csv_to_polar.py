import csv
import os


## PARAMETERS
prefix = "dhi_billing"

## DON'T EDIT BELOW ##

file_dir = os.path.dirname(__file__)
ifilename = os.path.join(file_dir, f"{prefix}.csv")
ofilename = f"{prefix}.polar"

ifile = open(ifilename, newline="")
ofile = open(ofilename, "w")

# Add base rules
base_str = f"""# Assume actions are hierarchical: R < RW < RWC < RWCU
allow_model(actor, "read", resource) :=
    allow_model(actor, "R", resource);
allow_model(actor, "write", resource) :=
    allow_model(actor, "RW", resource);
allow_model(actor, "create", resource) :=
    allow_model(actor, "RWC", resource);
allow_model(actor, "unlink", resource) :=
    allow_model(actor, "RWCU", resource);

allow_model(actor, "R", resource) :=
    allow_model(actor, "RW", resource);
allow_model(actor, "RW", resource) :=
    allow_model(actor, "RWC", resource);
allow_model(actor, "RWC", resource) :=
    allow_model(actor, "RWCU", resource);

# Lookup role for user
role(user, role) := user.groups.id = role;

# Top-level rules
allow_model(actor, action, resource) :=
    allow_{prefix}(actor, action, resource);

allow_{prefix}(actor, action, resource) :=
    role(actor, role),
    allow_{prefix}_by_role(role, action, resource);

# {prefix} Rules
"""

ofile.write(base_str)

reader = csv.DictReader(ifile, restkey="extra")
roles = set()
for row in reader:
    role = f'{row["group_id:id"]}'
    resource = f'{row["model_id:id"]}'
    if role == "" or resource == "":
        continue
    roles.add(role)
    if row["perm_unlink"] == "1":
        action = "RWCU"
    elif row["perm_create"] == "1":
        action = "RWC"
    elif row["perm_write"] == "1":
        action = "RW"
    elif row["perm_read"] == "1":
        action = "R"
    else:
        continue

    rule_str = f'allow_{prefix}_by_role("{role}","{action}","{resource}");\n'

    ofile.write(rule_str)

ofile.close()
ifile.close()

from polar import Polar, Predicate, Variable

p = Polar()
p.load(ofilename)

actions = ["R", "RW", "RWC", "RWCU"]
new_rules = []
for role in roles:
    for action in actions:
        q = f'allow_{prefix}_by_role("{role}", "{action}", model)'
        results = list(p._query_str(q))
        if len(results) > 0:
            new_rules.append({"role": role, "action": action, "resources": results})

print(new_rules)
print(len(new_rules))
