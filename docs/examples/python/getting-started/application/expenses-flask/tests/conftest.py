import pytest

from app import create_app


@pytest.fixture(scope="module")
def test_client():
    flask_app = create_app()
    flask_app.testing = True
    test_client = flask_app.test_client()
    ctx = flask_app.app_context()
    ctx.push()
    yield test_client
    ctx.pop()
