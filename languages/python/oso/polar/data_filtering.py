# Data Filtering

# We have some new stuff they have to define that is type and relationship information.
# We have some hooks they need to implement so we can fetch data.
# We have to have some top level call that initiates everything.

# Should I start with the user side, in the form of a test? Yes, that seems to be a great idea.
# And... Should that test use roles perhaps?????? Sure could.


# So we have some relationship types or whatever that's needed to get us some info
# We have the callback methods registered on something.
# We have the actuial evaluation of everything.

# Should the evaluation be like a different VM in the core? It sort of does the same
# thing maybe as the normal vm where it talks in events?

# Other ways to do it, maybe all the evaluation does happen in the host language but we
# just have to implement it for each language. It's not all that much maybe?

# Maybe it is the visitor pattern thing that we had before and you can fill out
# different parts of the expression?

# If the fetching is events, maybe then it all runs in the core and calls out?
# How would you do other apis if you did it like that though?

# Just going all python to start anyway so don't worry about it dawg just hack.

from typing import Any
from dataclasses import dataclass

VALID_KINDS = ["many-to-one"]


@dataclass
class Relationship:
    kind: str
    other_type: Any
    my_field: str
    other_field: str
