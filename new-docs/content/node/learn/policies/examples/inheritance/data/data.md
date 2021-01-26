---
actorClasses: |
    ```js
    class PatientData {
      constructor(patient) {
        this.patient = patient;
      }
    }
    oso.registerClass(PatientData);

    class Lab extends PatientData {}
    oso.registerClass(Lab);

    class Order extends PatientData {}
    oso.registerClass(Order);

    class Test extends PatientData {}
    oso.registerClass(Test);
    ```
---
