---
actorClasses: |
    ```ruby
    class PatientData
      attr_accessor :patient

      def initialize(patient:)
        @patient = patient
      end
    end
    OSO.register_class(PatientData)

    class Lab < PatientData; end
    OSO.register_class(Lab)

    class Order < PatientData; end
    OSO.register_class(Order)

    class Test < PatientData; end
    OSO.register_class(Test)
    ```
---
