# frozen_string_literal: true

require_relative 'lib/osohq/polar/version'

Gem::Specification.new do |spec|
  spec.name          = 'osohq-polar'
  spec.version       = Osohq::Polar::VERSION
  spec.authors       = ['Oso Security']
  spec.email         = ['support@osohq.com']

  spec.summary       = 'Polar language interface.'
  spec.homepage      = 'https://osohq.com/'
  spec.required_ruby_version = Gem::Requirement.new('>= 2.7.0')

  spec.metadata['homepage_uri'] = spec.homepage
  spec.metadata['source_code_uri'] = 'https://github.com/osohq/oso'
  spec.metadata['changelog_uri'] = 'https://github.com/osohq/oso/blob/master/CHANGELOG.md'

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(File.expand_path(__dir__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  end
  spec.bindir        = 'exe'
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']
end
