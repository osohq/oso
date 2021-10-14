import oso


@oso.polar_class
class User:
    def __init__(self, role, treated):
        self.role = role
        self._treated = treated

    def treated(self, patient):
        return patient in self._treated


# start-patient-data
@oso.polar_class
class PatientData:
    def __init__(self, patient):
        self.patient = patient


@oso.polar_class
class Lab(PatientData):
    pass


@oso.polar_class
class Order(PatientData):
    pass


@oso.polar_class
class Test(PatientData):
    pass
    # end-patient-data
