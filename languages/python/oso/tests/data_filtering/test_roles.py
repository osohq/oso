from helpers import *
from roles_examples import *

def test_roles_data_filtering_owner(oso, roles):
    check_authz(oso, leina, "invite", Org, [osohq])
    check_authz(oso, leina, "pull", Repo, [oso_repo, demo_repo])
    check_authz(oso, leina, "push", Repo, [oso_repo, demo_repo])
    check_authz(oso, leina, "edit", Issue, [oso_bug])


def test_roles_data_filtering_member(oso, roles):
    check_authz(oso, steve, "pull", Repo, [oso_repo, demo_repo])
    check_authz(oso, steve, "push", Repo, [])
    check_authz(oso, steve, "invite", Org, [])
    check_authz(oso, steve, "edit", Issue, [])


def test_roles_data_filtering_writer(oso, roles):
    check_authz(oso, gabe, "invite", Org, [])
    check_authz(oso, gabe, "pull", Repo, [oso_repo])
    check_authz(oso, gabe, "push", Repo, [oso_repo])
    check_authz(oso, gabe, "edit", Issue, [oso_bug])
