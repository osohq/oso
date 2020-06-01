@dataclass
class User:
    name: str = ''

    def role(self):
        yield from db.query('SELECT role FROM user_roles WHERE username = ?', [self.name])
