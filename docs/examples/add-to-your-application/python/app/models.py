from dataclasses import dataclass
from typing import List

@dataclass
class Repository:
    name: str

    def get_by_name(name):
        return repos_db[name]

repos_db = {
    "gmail": Repository("gmail")
}

# docs: start
@dataclass
class Role:
    name: str
    repository: Repository

@dataclass
class User:
    roles: List[Role]

users_db = {
    "larry": User([Role(name="admin",
                        repository=repos_db["gmail"])]),
}
# docs: end
