---
actorClasses: |
    ```python
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
    ```
---
