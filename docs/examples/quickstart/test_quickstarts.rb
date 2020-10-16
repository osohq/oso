# frozen_string_literal: true

require 'English'
require 'fileutils'
require 'timeout'

require 'bundler'

CURL_ERROR = "curl: (7) Failed to connect to localhost port 5050: Connection refused\n"

quickstarts = [
  { lang: 'python', setup: 'pip install -r requirements.txt', server: 'python server.py' },
  { lang: 'ruby', setup: 'bundle', server: 'bundle exec ruby server.rb' },
  { lang: 'rust', setup: 'cargo build', server: 'cargo run' },
  { lang: 'nodejs', setup: 'npm i', server: 'npm start' }
]

# TODO(gj): factor server polling into function

quickstarts.each do |qs|
  lang = qs[:lang]
  qs_dir = "oso-#{lang}-quickstart"
  Bundler.with_original_env do
    Dir.chdir(qs_dir) do
      prefix = "#{Time.now.to_i} [#{lang}]"
      puts "#{prefix} Installing dependencies..."
      setup_output = `#{qs[:setup]} 2>&1`
      raise "Setup step failed for #{lang.upcase}:\n#{setup_output}" unless $CHILD_STATUS.exitstatus.zero?

      Timeout.timeout 30 do
        begin
          puts "#{prefix} Starting server..."
          server = spawn qs[:server], %i[out err] => '/dev/null'
          received = CURL_ERROR
          while received == CURL_ERROR
            sleep 0.5
            received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/1 2>&1`
          end
          puts "#{prefix} Testing with no rules..."
          puts "#{prefix} Checking that Alice cannot see their own expense..."
          expected = "Not Authorized!\n"
          if received != expected
            raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
          end

          puts "#{prefix} Restarting server..."
          Process.kill 'TERM', server
          Process.wait server
          FileUtils.cp 'expenses.polar', 'original.polar'
          FileUtils.cp "../polar/expenses-01-#{lang}.polar", 'expenses.polar'
          server = spawn qs[:server], %i[out err] => '/dev/null'
          received = CURL_ERROR
          while received == CURL_ERROR
            sleep 0.5
            received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/3 2>&1`
          end
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
          Process.kill 'TERM', server
          Process.wait server
          FileUtils.cp "../polar/expenses-02-#{lang}.polar", 'expenses.polar'
          server = spawn qs[:server], %i[out err] => '/dev/null'
          received = CURL_ERROR
          while received == CURL_ERROR
            sleep 0.5
            received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/1 2>&1`
          end

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
          Process.kill 'TERM', server
          Process.wait server
          FileUtils.mv 'original.polar', 'expenses.polar', force: true
        end
      end
    end
  end
end
