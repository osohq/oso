from .test_duplicate_models import Base
from sqlalchemy import Column, Integer, Boolean


class Post(Base):
    __tablename__ = "posts_one"

    id = Column(Integer, primary_key=True)
    admin = Column(Boolean, nullable=False)
