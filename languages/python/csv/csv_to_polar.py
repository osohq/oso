import os
import pandas as pd

from polar import Polar, Predicate


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

# Top-level rules
allow_model(actor, action, resource) :=
    allow_{prefix}(actor, action, resource);

allow_{prefix}(actor, action, resource) :=
    role(actor, role),
    allow_{prefix}_by_role(role, action, resource);

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

p = Polar()
p.load(ofilename)


assert p._query_pred(
    Predicate(name="allow_model", args=[hr, "unlink", "model_dhi_insurance_card"])
).success

assert p._query_pred(
    Predicate(name="allow_model", args=[nurse, "create", "model_dhi_bill"])
).success

assert not p._query_pred(
    Predicate(name="allow_model", args=[nurse, "write", "model_dhi_fee_schedule"])
).success

assert p._query_pred(
    Predicate(
        name="allow_model",
        args=[receptionist, "create", "model_dhi_payment_details_line"],
    )
).success

assert not p._query_pred(
    Predicate(
        name="allow_model", args=[receptionist, "create", "model_dhi_fee_schedule"],
    )
).success

