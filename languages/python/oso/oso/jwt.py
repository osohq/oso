from authlib.jose import jwt
from typing import List

# Global JWT decode keys
# TODO: make this not global
JWT_DECODE_KEYS: List[str] = []


class Jwt:
    """ Takes in a jwt and exposes the attributes as a dictionary"""

    def __init__(self, token):
        self.token = token
        self.attribs = None

        for key in JWT_DECODE_KEYS:
            try:
                claims = jwt.decode(token, key)
                self.attribs = dict(claims)
                break
            except:
                pass

    @classmethod
    def add_key(cls, key):
        global JWT_DECODE_KEYS
        JWT_DECODE_KEYS.append(key)

    @classmethod
    def clear_keys(cls):
        global JWT_DECODE_KEYS
        JWT_DECODE_KEYS.clear()

    def attributes(self):
        if self.attribs:
            return self.attribs
