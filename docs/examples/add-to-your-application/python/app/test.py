import pytest

from .oso import oso
from .routes import app

def test_policy_loads_and_oso_inits():
    assert oso

def test_route_works():
    with app.test_client() as c:
        assert c.get("/repo/gmail").status_code == 200

def test_data_filtering_works():
    from .data_filtering import oso, Repository, Session

    repo = Repository(name="gmail")
    session = Session()
    session.add(repo)
    session.commit()

    with app.test_client() as c:
        response = c.get("/repos")
        assert response.status_code == 200
        # Ensure response contains at least some repo
        # object.
        assert len(response.data) > 2
