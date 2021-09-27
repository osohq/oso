from pathlib import Path

from oso import Oso

from .models import User, Repository

# Initialize the Oso object. This object is usually
# used globally throughout an application.
oso = Oso()

# Tell Oso about the data that you will authorize.
# These types can be referenced in the policy.
oso.register_class(User)
oso.register_class(Repository)

# Load your policy file.
oso.load_files([Path(__file__).parent / "main.polar"])
