import pytest

from sqlalchemy.orm import Session
from sqlalchemy import Column, Integer
from sqlalchemy_oso.flask import AuthorizedSQLAlchemy

from .models import Post, ModelBase

flask = pytest.importorskip("flask")
flask_sqlalchemy = pytest.importorskip("flask_sqlalchemy")


@pytest.fixture
def db_uri(tmp_path):
    tempfile = tmp_path / "db.sqlite"
    return f"sqlite:///{tempfile}"


@pytest.fixture
def flask_app(db_uri):
    app = flask.Flask(__name__)
    app.config["SQLALCHEMY_DATABASE_URI"] = db_uri
    return app


@pytest.fixture
def ctx(flask_app):
    with flask_app.app_context() as ctx:
        yield ctx


@pytest.fixture
def sqlalchemy(flask_app, oso):
    sqlalchemy = AuthorizedSQLAlchemy(
        get_oso=lambda: oso, get_user=lambda: "user", get_action=lambda: "read"
    )
    sqlalchemy.init_app(flask_app)
    return sqlalchemy


def test_authorized_sqlalchemy(ctx, flask_app, oso, sqlalchemy, post_fixtures):
    oso.load_str('allow("user", "read", post: Post) if post.id = 0;')
    sqlalchemy.init_app(flask_app)
    engine = sqlalchemy.get_engine()
    ModelBase.metadata.create_all(engine)

    sessionmaker = sqlalchemy.create_session({})

    # Create fixtures.
    fixture_session = sessionmaker()
    post_fixtures(fixture_session)
    fixture_session.commit()
    assert Session(bind=engine).query(Post).count() > 0

    authorized_session = sessionmaker()

    assert authorized_session.query(Post).count() == 1

    with flask_app.app_context():
        assert sqlalchemy.session.query(Post).count() == 1


def test_flask_model(ctx, flask_app, oso, sqlalchemy):
    class TestModel(sqlalchemy.Model):
        id = Column(Integer, primary_key=True)

    sqlalchemy.create_all()
    sqlalchemy.session.add(TestModel(id=1))
    sqlalchemy.session.add(TestModel(id=2))
    sqlalchemy.session.commit()

    oso.register_class(TestModel)

    oso.load_str("allow(_, _, tm: TestModel) if tm.id = 1;")

    authorized = sqlalchemy.session.query(TestModel).all()
    assert len(authorized) == 1
    assert authorized[0].id == 1

    authorized = TestModel.query.all()
    assert len(authorized) == 1
    assert authorized[0].id == 1

    oso.load_str("allow(_, _, tm: TestModel) if tm.id = 2;")

    authorized = TestModel.query.all()
    assert len(authorized) == 2
    assert authorized[0].id == 1
    assert authorized[1].id == 2
