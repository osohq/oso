OSO ||= Oso.new

class User
  attr_accessor :role

  def initialize(role:, treated:)
    @role = role
    @_treated = treated
  end

  def treated(patient)
    @_treated.include?(patient)
  end
end

OSO.register_class(User)

# start-patient-data
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
# end-patient-data
