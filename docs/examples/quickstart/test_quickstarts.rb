# frozen_string_literal: true

require 'English'
require 'fileutils'
require 'timeout'

require 'bundler'

CURL_ERROR = "curl: (7) Failed to connect to localhost port 5000: Connection refused\n"
CURL_EMPTY = "curl: (52) Empty reply from server\n"

quickstarts = [
  { lang: 'go', setup: 'go build', server: './oso-go-quickstart' },
  { lang: 'java', setup: 'make install', server: 'make run' },
  { lang: 'nodejs', setup: 'npm i', server: 'npm run dev' },
  { lang: 'python', setup: 'pip install --upgrade -r requirements.txt', server: 'FLASK_APP=app.server python -m flask run' },
  { lang: 'ruby', setup: 'bundle', server: 'bundle exec ruby server.rb' },
  # { lang: 'rust', setup: 'cargo build', server: 'cargo run' }
]

def start_server(server, repo)
  server = spawn server
  received = CURL_ERROR
  while [CURL_ERROR, CURL_EMPTY].include? received
    sleep 0.5
    received = `curl -sS localhost:5000/repo/#{repo} 2>&1`
  end
  [server, received]
end

def ensure_port_5000_is_open
  until (server = `lsof -ti :5000 2>&1`.split.first.to_i).zero?
    sleep 0.5
    Process.kill 'TERM', server
  end
rescue Errno::ESRCH => e
  puts "#{e}: #{server}"
end

def kill_server(server)
  Process.kill 'TERM', server
  Process.wait2 server
rescue Errno::ESRCH => e
  puts "#{e}: #{server}"
end

# rubocop:disable Metrics/BlockLength

quickstarts.each do |qs|
  lang = qs[:lang]
  qs_dir = File.join(File.expand_path(__dir__), lang)
  Bundler.with_unbundled_env do
    Dir.chdir(qs_dir) do
      prefix = "#{Time.now.to_i} [#{lang}]"
      puts "#{prefix} Installing dependencies..."
      setup_output = `#{qs[:setup]} 2>&1`
      raise "Setup step failed for #{lang.upcase}:\n#{setup_output}" unless $CHILD_STATUS.exitstatus.zero?

      Timeout.timeout 15 do
        begin
          ensure_port_5000_is_open

          puts "#{prefix} Starting server..."
          server, received = start_server qs[:server], "gmail"
          puts "#{prefix} Checking that /repo/gmail returns a 200..."
          expected = 'Welcome to repo gmail'
          unless received.include?(expected)
            raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
          end

          puts "#{prefix} Checking that /repo/react returns a 404..."
          received = `curl -sS localhost:5000/repo/react 2>&1`
          expected = 'Repo named react was not found'
          unless received.include?(expected)
            raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
          end

          puts "#{prefix} Success!"
        ensure
          kill_server server unless server.nil?
          ensure_port_5000_is_open
        end
      end
    end
  end
end

# rubocop:enable Metrics/BlockLength
