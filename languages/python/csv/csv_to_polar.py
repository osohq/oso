import os
import pandas as pd

## PARAMETERS
prefix = "dhi_billing"

## DON'T EDIT BELOW ##

file_dir = os.path.dirname(__file__)
ifilename = os.path.join(file_dir, f"{prefix}.csv")
ofilename = f"{prefix}.polar"

ifile = open(ifilename, newline="")
ofile = open(ofilename, "w")

# Add base rules
base_str = f"""
# Top-level rules
allow(actor, action, resource) := allow_model(actor, action, resource);

allow_model(actor, action, resource) :=
    allow_{prefix}(actor, action, resource);

allow_{prefix}(actor, action, resource) :=
    role(actor, role),
    allow_{prefix}_by_role(role, action, resource);

# Assume actions are hierarchical: R < RW < RWC < RWCU
allow_model(actor, "read", resource) :=
    allow_model(actor, "R", resource)
	| allow_model(actor, "RW", resource)
	| allow_model(actor, "RWC", resource)
	| allow_model(actor, "RWCU", resource);

allow_model(actor, "write", resource) :=
    allow_model(actor, "RW", resource)
	| allow_model(actor, "RWC", resource)
	| allow_model(actor, "RWCU", resource);

allow_model(actor, "create", resource) :=
    allow_model(actor, "RWC", resource)
	| allow_model(actor, "RWCU", resource);

allow_model(actor, "unlink", resource) :=
    allow_model(actor, "RWCU", resource);

# Lookup role for user
role(user, role) := group in user.groups, group.id = role;


## {prefix} Rules
"""

ofile.write(base_str)

df = pd.read_csv(ifile).dropna()
df = df.drop(["name", "id"], axis="columns")

roles = df["group_id:id"].unique()
actions = ["perm_unlink", "perm_create", "perm_write", "perm_read"]
action_map = {
    "perm_unlink": "RWCU",
    "perm_create": "RWC",
    "perm_write": "RW",
    "perm_read": "R",
}

for role in roles:
    ofile.write(f"\n# {role}\n")
    rdf = df[df["group_id:id"] == role]
    for action in actions:
        radf = rdf[rdf[action] == 1]
        rdf = rdf.drop(radf.index)
        models = list(map(lambda x: f'"{x}"', radf["model_id:id"]))
        if not models:
            continue
        elif len(models) == 1:
            rule_body = f"resource = {models[0]}"
        else:
            models = (",\n\t\t").join(models)
            rule_body = f"resource in [\n\t\t{models}\n\t]"

        rule_str = f'allow_{prefix}_by_role("{role}", "{action_map[action]}", resource) := \n\t{rule_body};\n'
        ofile.write(rule_str)

ifile.close()
ofile.close()


### TESTING
from oso import Oso, Predicate


class Group:
    def __init__(self, id):
        self.id = id


class User:
    def __init__(self, groups):
        self.groups = groups


hr = User([Group("user_access.dhi_group_hr")])
nurse = User([Group("user_access.dhi_group_nurse")])
billing = User([Group("user_access.dhi_group_billing")])
receptionist = User([Group("user_access.dhi_group_receptionist")])

policy = Oso()
policy.load(ofilename)


assert policy.allow(hr, "unlink", "model_dhi_insurance_card")
assert policy.allow(nurse, "create", "model_dhi_bill")
assert not policy.allow(nurse, "write", "model_dhi_fee_schedule")
assert policy.allow(receptionist, "create", "model_dhi_payment_details_line")
assert not policy.allow(receptionist, "create", "model_dhi_fee_schedule")


### TIMING
import timeit


setup = """
from oso import Oso, Predicate
class Group:
    def __init__(self, id):
        self.id = id
class User:
    def __init__(self, groups):
        self.groups = groups
hr = User([Group("user_access.dhi_group_hr")])
nurse = User([Group("user_access.dhi_group_nurse")])
billing = User([Group("user_access.dhi_group_billing")])
receptionist = User([Group("user_access.dhi_group_receptionist")])
policy = Oso()
policy.load("dhi_billing.polar")
"""

print(
    timeit.timeit(
        'policy.allow(hr, "unlink", "model_dhi_insurance_card")', setup=setup, number=1,
    )
)
