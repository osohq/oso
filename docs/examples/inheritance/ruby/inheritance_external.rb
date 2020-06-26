class Actor
  attr_accessor :role

  def initialize(role:, treated:)
    @role = role
    @_treated = treated
  end

  def treated(patient)
    @_treated.include?(patient)
  end
end

## START MARKER ##
class PatientData
  attr_accessor :patient

  def initialize(patient:)
    @patient = patient
  end
end

class Lab < PatientData
end

class Order < PatientData
end

class Test < PatientData
end

def setup(oso)
  oso.register_class(Actor)
  oso.register_class(PatientData)
  oso.register_class(Lab)
  oso.register_class(Order)
  oso.register_class(Test)
end
