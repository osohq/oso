from sqlalchemy import create_engine
from sqlalchemy.orm import Session
from oso import Oso
from sqlalchemy_oso import authorized_sessionmaker, register_models
from sqlalchemy_example.models import *

oso = Oso()
register_models(oso, Model)
oso.load_file("sqlalchemy_example/policy.polar")

# setup some test data

user = User(username='user')
manager = User(username='manager', manages=[user])

public_user_post = Post(contents='public_user_post',
                        access_level='public',
                        created_by=user)
private_user_post = Post(contents='private_user_post',
                        access_level='private',
                        created_by=user)
private_manager_post = Post(contents='private_manager_post',
                            access_level='private',
                            created_by=manager)
public_manager_post = Post(contents='public_manager_post',
                           access_level='public',
                           created_by=manager)

# and load that data into SQLAlchemy:
engine = create_engine('sqlite:///:memory:')
Model.metadata.create_all(engine)
session = Session(bind=engine)
session.add_all([user, manager, public_user_post, private_user_post, private_manager_post, public_manager_post])
session.commit()

# Now that we've setup some test data, let's use oso to authorize Posts that
# User(username="user") can see.

AuthorizedSession = authorized_sessionmaker(bind=engine,
                                            get_oso=lambda: oso,
                                            get_user=lambda: user,
                                            get_action=lambda: "read")
session = AuthorizedSession()

posts = session.query(Post).all()

# user can see their own public and private posts, and other public posts.
# ['public_user_post', 'private_user_post', 'public_manager_post']
print("readable posts: ", [p.contents for p in posts])

# Now we'll authorize access to manager's Posts.
AuthorizedSession = authorized_sessionmaker(bind=engine,
                                            get_oso=lambda: oso,
                                            get_user=lambda: manager,
                                            get_action=lambda: "read")
session = AuthorizedSession()

# Now we'll authorize access to manager's Posts.
posts = session.query(Post).all()

# We got 4 posts this time, the manager's public and private posts, other
# user's private posts and private posts of users that the manager user
# manages.
# ['public_user_post', 'private_user_post', 'private_manager_post', 'public_manager_post']
print("readable posts: ", [p.contents for p in posts])

# 'user'
print("manager manages: ", manager.manages[0].username)

# More complex queries can be authorized as well.
AuthorizedSession = authorized_sessionmaker(bind=engine,
                                            get_oso=lambda: oso,
                                            get_user=lambda: user,
                                            get_action=lambda: "read")
session = AuthorizedSession()

posts = session.query(Post.contents).all()
print("readable posts: ", [p.contents for p in posts])

# [('public_user_post', 'user'),
#  ('public_user_post', 'manager'),
#  ('private_user_post', 'user'),
#  ('private_user_post', 'manager'),
#  ('public_manager_post', 'user'),
#  ('public_manager_post', 'manager')]
data = session.query(Post.contents, User.username).all()
print("More complex queries with Post and Users", data)

# [('public_user_post', 'user'),
#  ('private_user_post', 'user'),
#  ('public_manager_post', 'manager')]
data = session.query(Post.contents, User.username).join(User).all()
print("More complex queries with Post and Users", data)

