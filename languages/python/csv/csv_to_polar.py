import os
import pandas as pd


def get_polar_raw(prefix):
    pass


def to_polar_consolidated(prefix):
    # Get files
    (ifile, ofile) = get_files(prefix)

    # write base rules to file
    write_base_rules(ofile, prefix)

    # read csv file into DataFrame
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

    # for each unique role-permissions combination, write a rule over all applicable models
    for role in roles:
        ofile.write(f"\n# {role}\n")
        rdf = df[df["group_id:id"] == role]  # df for this role

        for action in actions:
            radf = rdf[rdf[action] == 1]  # df for this role-action pair
            rdf = rdf.drop(radf.index)

            # get all applicable models
            models = list(map(lambda x: f'"{x}"', radf["model_id:id"]))
            if not models:
                continue
            elif len(models) == 1:
                rule_body = f"resource = {models[0]}"
            else:
                models = (",\n\t\t").join(models)
                rule_body = f"resource in [\n\t\t{models}\n\t]"

            # write rule_str to file
            rule_str = f'allow_{prefix}_by_role("{role}", "{action_map[action]}", resource) := \n\t{rule_body};\n'
            ofile.write(rule_str)

    ifile.close()
    ofile.close()


def get_files(prefix):
    # Get files
    file_dir = os.path.dirname(__file__)
    ifilename = os.path.join(file_dir, f"{prefix}.csv")
    ofilename = f"{prefix}.polar"

    ifile = open(ifilename, newline="")
    ofile = open(ofilename, "w")

    return (ifile, ofile)


def write_base_rules(ofile, prefix):
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
