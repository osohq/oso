# frozen_string_literal: true

require_relative 'lib/oso/version'

Gem::Specification.new do |spec|
  spec.name          = 'oso-oso'
  spec.version       = Oso::VERSION
  spec.authors       = ['Oso Security']
  spec.email         = ['support@osohq.com']

  spec.summary       = 'Oso authorization API.'
  spec.homepage      = 'https://www.osohq.com/'
  spec.required_ruby_version = Gem::Requirement.new('>= 2.4.0')

  spec.metadata['homepage_uri'] = spec.homepage
  spec.metadata['source_code_uri'] = 'https://github.com/osohq/oso'

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(File.expand_path(__dir__)) do
    files = `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
    files + Dir['ext/oso-oso/lib/*']
  end

  spec.bindir        = 'bin'
  spec.executables   = spec.files.grep(%r{^bin/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']

  # Runtime dependencies
  spec.add_runtime_dependency 'ffi', '~> 1.0'

  # Development dependencies
  spec.add_development_dependency 'pry-byebug', '~> 3.9.0'
  spec.add_development_dependency 'rake', '~> 12.0'
  spec.add_development_dependency 'rspec', '~> 3.0'
  spec.add_development_dependency 'solargraph', '~> 0.39.8'
  spec.add_development_dependency 'yard', '~> 0.9.25'
end
