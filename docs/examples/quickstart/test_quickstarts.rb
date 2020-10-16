# frozen_string_literal: true

require 'English'
require 'fileutils'
require 'timeout'

CURL_ERROR = "curl: (7) Failed to connect to localhost port 5050: Connection refused\n"

quickstarts = [
  { lang: 'ruby', setup: 'bundle', server: 'bundle exec ruby server.rb' },
  { lang: 'nodejs', setup: 'npm install', server: 'npm start' },
  { lang: 'python', setup: 'pip install -r requirements.txt', server: 'python server.py' },
  { lang: 'rust', setup: 'cargo build', server: 'cargo run' }
]

# TODO(gj): factor server polling into function

quickstarts.each do |qs|
  lang = qs[:lang]
  qs_dir = "oso-#{lang}-quickstart"
  Dir.chdir(qs_dir) do
    puts "[#{lang}] Installing dependencies..."
    setup_output = `#{qs[:setup]} 2>&1`
    raise "Setup step failed for #{lang.upcase}:\n#{setup_output}" unless $CHILD_STATUS.exitstatus.zero?

    Timeout.timeout 60 do
      begin
        puts "[#{lang}] Starting server..."
        server = spawn qs[:server], %i[out err] => '/dev/null'
        received = CURL_ERROR
        while received == CURL_ERROR
          sleep 0.5
          received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/1 2>&1`
        end
        puts "[#{lang}] Testing with no rules..."
        puts "[#{lang}] Checking that Alice cannot see their own expense..."
        expected = "Not Authorized!\n"
        if received != expected
          raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
        end

        puts "[#{lang}] Restarting server..."
        Process.kill 'INT', server
        Process.wait server
        FileUtils.cp 'expenses.polar', 'original.polar'
        FileUtils.cp "../polar/expenses-01-#{lang}.polar", 'expenses.polar'
        server = spawn qs[:server], %i[out err] => '/dev/null'
        received = CURL_ERROR
        while received == CURL_ERROR
          sleep 0.5
          received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/3 2>&1`
        end
        puts "[#{lang}] Testing string matching rule..."
        puts "[#{lang}] Checking that alice@example.com can see any expense..."
        ['Expense', '50000', 'flight', 'bhavik@example.com'].each do |text|
          unless received.include? text
            raise "#{lang.upcase} failure\n\texpected output to contain: #{text}\n\treceived: #{received.inspect}\n"
          end
        end
        puts "[#{lang}] Checking that alice@foo.bar cannot see any expense..."
        received = `curl -sSH "user: alice@foo.bar" localhost:5050/expenses/1 2>&1`
        expected = "Not Authorized!\n"
        if received != expected
          raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
        end

        puts "[#{lang}] Restarting server..."
        Process.kill 'INT', server
        Process.wait server
        FileUtils.cp "../polar/expenses-02-#{lang}.polar", 'expenses.polar'
        server = spawn qs[:server], %i[out err] => '/dev/null'
        received = CURL_ERROR
        while received == CURL_ERROR
          sleep 0.5
          received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/1 2>&1`
        end

        puts "[#{lang}] Testing application data rule..."
        puts "[#{lang}] Checking that Alice can see their own expense..."
        ['Expense', '500', 'coffee', 'alice@example.com'].each do |text|
          unless received.include? text
            raise "#{lang.upcase} failure\n\texpected output to contain: #{text}\n\treceived: #{received.inspect}\n"
          end
        end

        puts "[#{lang}] Checking that Bhavik can see their own expense..."
        received = `curl -sSH "user: bhavik@example.com" localhost:5050/expenses/3 2>&1`
        ['Expense', '50000', 'flight', 'bhavik@example.com'].each do |text|
          unless received.include? text
            raise "#{lang.upcase} failure\n\texpected output to contain: #{text}\n\treceived: #{received.inspect}\n"
          end
        end

        puts "[#{lang}] Checking that Alice cannot see Bhavik's expense..."
        received = `curl -sSH "user: alice@example.com" localhost:5050/expenses/3 2>&1`
        expected = "Not Authorized!\n"
        if received != expected
          raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
        end

        puts "[#{lang}] Checking that Bhavik cannot see Alice's expense..."
        received = `curl -sSH "user: bhavik@example.com" localhost:5050/expenses/1 2>&1`
        expected = "Not Authorized!\n"
        if received != expected
          raise "#{lang.upcase} failure\n\texpected: #{expected.inspect}\n\treceived: #{received.inspect}\n"
        end

        puts "[#{lang}] Success!"
      ensure
        Process.kill 'INT', server
        Process.wait server
        FileUtils.mv 'original.polar', 'expenses.polar', force: true
      end
    end
  end
end
