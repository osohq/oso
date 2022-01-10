import oso


class User:
    def __init__(self, role, treated):
        self.role = role
        self._treated = treated

    def treated(self, patient):
        return patient in self._treated


# start-patient-data
class PatientData:
    def __init__(self, patient):
        self.patient = patient


class Lab(PatientData):
    pass


class Order(PatientData):
    pass


class Test(PatientData):
    pass
    # end-patient-data
