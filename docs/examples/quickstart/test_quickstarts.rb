# frozen_string_literal: true

require 'English'
require 'fileutils'
require 'timeout'

require 'bundler'

CURL_ERROR = "curl: (7) Failed to connect to localhost port 5050: Connection refused\n"
CURL_EMPTY = "curl: (52) Empty reply from server\n"

quickstarts = [
  { lang: 'java', setup: 'make build', server: 'make run' },
  { lang: 'nodejs', setup: 'npm i', server: 'npm start' },
  { lang: 'python', setup: 'pip install --upgrade -r requirements.txt', server: 'python server.py' },
  { lang: 'ruby', setup: 'bundle', server: 'bundle exec ruby server.rb' },
  { lang: 'rust', setup: 'cargo build --target-dir ../../../../target', server: 'cargo run' }
]

def start_server(server, user, expense_id)
  server = spawn server
  received = CURL_ERROR
  while [CURL_ERROR, CURL_EMPTY].include? received
    sleep 0.5
    received = `curl -sSH "user: #{user}" localhost:5050/expenses/#{expense_id} 2>&1`
  end
  [server, received]
end

def ensure_port_5050_is_open
  until (server = `lsof -ti :5050 2>&1`.split.first.to_i).zero?
    sleep 0.5
    Process.kill 'TERM', server
  end
rescue Errno::ESRCH => e
  puts "#{e}: #{server}"
end

def kill_server(server)
  return if server.nil?

  Process.kill 'TERM', server
  Process.wait2 server
rescue Errno::ESRCH => e
  puts "#{e}: #{server}"
end

# rubocop:disable Metrics/BlockLength

quickstarts.each do |qs|
  lang = qs[:lang]
  qs_dir = "oso-#{lang}-quickstart"
  Bundler.with_unbundled_env do
    Dir.chdir(qs_dir) do
      prefix = "#{Time.now.to_i} [#{lang}]"
      puts "#{prefix} Installing dependencies..."
      setup_output = `#{qs[:setup]} 2>&1`
      raise "Setup step failed for #{lang.upcase}:\n#{setup_output}" unless $CHILD_STATUS.exitstatus.zero?

      Timeout.timeout 30 do
        begin
          ensure_port_5050_is_open

          puts "#{prefix} Starting server..."
          server, received = start_server qs[:server], 'alice@example.com', 1
          puts "#{prefix} Testing with no rules..."
          puts "#{prefix} Checking that Alice cannot see their own expense..."
          expected = "Not Authorized!\n"
          if received != expected
            raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
          end

          puts "#{prefix} Restarting server..."
          kill_server server
          ensure_port_5050_is_open

          FileUtils.cp 'expenses.polar', 'original.polar'
          FileUtils.cp "../polar/expenses-01-#{lang}.polar", 'expenses.polar'

          server, received = start_server qs[:server], 'alice@example.com', 3
          puts "#{prefix} Testing string matching rule..."
          puts "#{prefix} Checking that alice@example.com can see any expense..."
          ['Expense', '50000', 'flight', 'bhavik@example.com'].each do |text|
            unless received.include? text
              raise "#{lang.upcase} failure\n\texpected output to contain: #{text}\n\treceived: #{received.inspect}\n"
            end
          end
          puts "#{prefix} Checking that alice@foo.bar cannot see any expense..."
          received = `curl -sSH "user: alice@foo.bar" localhost:5050/expenses/1 2>&1`
          expected = "Not Authorized!\n"
          if received != expected
            raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
          end

          puts "#{prefix} Restarting server..."
          kill_server server
          ensure_port_5050_is_open

          FileUtils.cp "../polar/expenses-02-#{lang}.polar", 'expenses.polar'

          server, received = start_server qs[:server], 'alice@example.com', 1
          puts "#{prefix} Testing application data rule..."
          puts "#{prefix} Checking that Alice can see their own expense..."
          ['Expense', '500', 'coffee', 'alice@example.com'].each do |text|
            unless received.include? text
              raise "#{lang.upcase} failure\n\texpected output to contain: #{text}\n\treceived: #{received.inspect}\n"
            end
          end

          puts "#{prefix} Checking that Bhavik can see their own expense..."
          received = `curl -sSH "user: bhavik@example.com" localhost:5050/expenses/3 2>&1`
          ['Expense', '50000', 'flight', 'bhavik@example.com'].each do |text|
            unless received.include? text
              raise "#{lang.upcase} failure\n\texpected output to contain: #{text}\n\treceived: #{received.inspect}\n"
            end
          end

          puts "#{prefix} Checking that Alice cannot see Bhavik's expense..."
          received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/3 2>&1`
          expected = "Not Authorized!\n"
          if received != expected
            raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
          end

          puts "#{prefix} Checking that Bhavik cannot see Alice's expense..."
          received = `curl -sSH "user: bhavik@example.com" localhost:5050/expenses/1 2>&1`
          expected = "Not Authorized!\n"
          if received != expected
            raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
          end

          puts "#{prefix} Success!"
        ensure
          kill_server server
          ensure_port_5050_is_open
          FileUtils.mv 'original.polar', 'expenses.polar', force: true
        end
      end
    end
  end
end

# rubocop:enable Metrics/BlockLength
