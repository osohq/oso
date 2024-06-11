# frozen_string_literal: true

require_relative 'lib/oso/version'

Gem::Specification.new do |spec|
  spec.name          = 'oso-oso'
  spec.version       = Oso::VERSION
  spec.authors       = ['Oso Security, Inc.']
  spec.email         = ['support@osohq.com']
  spec.licenses      = ['Apache-2.0']
  spec.summary       = 'We have deprecated the legacy Oso open source library. We have plans for the next open ' \
                       "source release and we're looking forward to getting feedback from the community leading up " \
                       'to that point (please reach out to us in the Slack #help channel). In the meantime, if ' \
                       "you're happy using the Oso open source library now, nothing needs to change – i.e., we are " \
                       "not end-of-lifing (EOL) the library and we'll continue to provide support and critical bug " \
                       'fixes. More context: [here](https://www.osohq.com/docs/oss/getting-started/deprecation.html).'
  spec.homepage      = 'https://www.osohq.com/'

  spec.required_ruby_version = Gem::Requirement.new('>= 2.6.10')

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
  spec.add_development_dependency 'activerecord'
  spec.add_development_dependency 'pry-byebug', '~> 3.10.1'
  spec.add_development_dependency 'rake', '~> 12.0'
  spec.add_development_dependency 'rspec', '~> 3.0'
  spec.add_development_dependency 'rubocop', '~> 1.58.0'
  spec.add_development_dependency 'solargraph', '~> 0.49.0'
  spec.add_development_dependency 'sqlite3'
  spec.add_development_dependency 'yard', '~> 0.9.25'
end
