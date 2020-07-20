from oso import Oso

oso = Oso()

# Whatever classes are necessesary for the tests go here.
@oso.register_class
class A:
    def __init__(self, x):
        self.x = x


oso.load_file("test.polar")
oso._load_queued_files()
print("Tests Pass")
