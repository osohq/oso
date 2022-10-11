from sqlalchemy import Boolean, Column, Integer

from .test_duplicate_models import Base


class Post(Base):
    __tablename__ = "posts_one"

    id = Column(Integer, primary_key=True)
    admin = Column(Boolean, nullable=False)
