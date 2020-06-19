from oso import Oso, Predicate
import timeit
from csv_to_polar import to_polar_consolidated


class Group:
    def __init__(self, id):
        self.id = id


class User:
    def __init__(self, groups):
        self.groups = groups


def main():
    to_polar_consolidated(prefix="dhi_billing")

    hr = User([Group("user_access.dhi_group_hr")])
    nurse = User([Group("user_access.dhi_group_nurse")])
    billing = User([Group("user_access.dhi_group_billing")])
    receptionist = User([Group("user_access.dhi_group_receptionist")])

    policy = Oso()
    policy.load("dhi_billing.polar")

    assert policy.allow(hr, "unlink", "model_dhi_insurance_card")
    assert policy.allow(nurse, "create", "model_dhi_bill")
    assert not policy.allow(nurse, "write", "model_dhi_fee_schedule")
    assert policy.allow(receptionist, "create", "model_dhi_payment_details_line")
    assert not policy.allow(receptionist, "create", "model_dhi_fee_schedule")

    time_allow()


def time_allow():
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
            'policy.allow(hr, "unlink", "model_dhi_insurance_card")',
            setup=setup,
            number=1,
        )
    )


if __name__ == "__main__":
    main()
