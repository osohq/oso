```python
from sqlalchemy import create_engine
from sqlalchemy.orm import Session
from oso import Oso
from sqlalchemy_oso.hooks import authorized_sessionmaker
from sqlalchemy_oso.auth import register_models
from sqlalchemy_example.models import *
```


```python
oso = Oso()
register_models(oso, Model)
oso.load_file("sqlalchemy_example/policy.polar")
```


```python
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

```


```python
engine = create_engine('sqlite:///:memory:')
Model.metadata.create_all(engine)
session = Session(bind=engine)
session.add_all([user, manager, public_user_post, private_user_post, private_manager_post, public_manager_post])
session.commit()
```


```python
AuthorizedSession = authorized_sessionmaker(bind=engine,
                                            get_oso=lambda: oso,
                                            get_user=lambda: user,
                                            get_action=lambda: "read")
```


```python
session = AuthorizedSession()
```

Now that we've setup some test data, let's use **oso** to authorize `Post`s that `User(username="user")` can see.


```python
posts = session.query(Post).all()
```


```python
[p.contents for p in posts]
```




    ['public_user_post', 'private_user_post', 'public_manager_post']



`user` can see their own public and private posts, and other public posts.


```python
AuthorizedSession = authorized_sessionmaker(bind=engine,
                                            get_oso=lambda: oso,
                                            get_user=lambda: manager,
                                            get_action=lambda: "read")
```


```python
session = AuthorizedSession()
```

Now we'll authorize access to `manager`'s `Post`s.


```python
posts = session.query(Post).all()
```


```python
[p.contents for p in posts]
```




    ['public_user_post',
     'private_user_post',
     'private_manager_post',
     'public_manager_post']



We got 4 posts this time, the manager's public and private posts, other user's private posts **and** private posts of users that the `manager` user manages.


```python
manager.manages[0].username
```




    'user'



More complex queries can be authorized as well.


```python
AuthorizedSession = authorized_sessionmaker(bind=engine,
                                            get_oso=lambda: oso,
                                            get_user=lambda: user,
                                            get_action=lambda: "read")
session = AuthorizedSession()
```


```python
session.query(Post.contents).all()
```




    [('public_user_post'), ('private_user_post'), ('public_manager_post')]




```python
session.query(Post.contents, User.username).all()
```




    [('public_user_post', 'user'),
     ('public_user_post', 'manager'),
     ('private_user_post', 'user'),
     ('private_user_post', 'manager'),
     ('public_manager_post', 'user'),
     ('public_manager_post', 'manager')]




```python
session.query(Post.contents, User.username).join(User).all()
```




    [('public_user_post', 'user'),
     ('private_user_post', 'user'),
     ('public_manager_post', 'manager')]




```python

```
