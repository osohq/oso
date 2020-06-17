# frozen_string_literal: true

require_relative 'lib/osohq/oso/version'

Gem::Specification.new do |spec|
  spec.name          = 'osohq-oso'
  spec.version       = Osohq::Oso::VERSION
  spec.authors       = ['Oso Security']
  spec.email         = ['support@osohq.com']

  spec.summary       = 'Oso authorization interface.'
  spec.homepage      = 'https://www.osohq.com/'
  spec.required_ruby_version = Gem::Requirement.new('>= 2.7.0')

  spec.metadata['homepage_uri'] = spec.homepage

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(File.expand_path(__dir__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  end
  spec.bindir        = 'exe'
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']

  # # Runtime dependencies
  # spec.add_runtime_dependency 'osohq-polar', '~> 0.1'

  # Development dependencies
  spec.add_development_dependency 'pry-byebug', '~> 3.9.0'
  spec.add_development_dependency 'rake', '~> 12.0'
  spec.add_development_dependency 'rspec', '~> 3.0'
  spec.add_development_dependency 'solargraph', '~> 0.39.8'
end
