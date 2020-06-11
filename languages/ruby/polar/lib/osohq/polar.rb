# frozen_string_literal: true

require 'json'

require 'osohq/polar/version'
require 'osohq/polar/ffi'

module Osohq
  module Polar
    module Error
      class FreeError < ::RuntimeError; end
      class UnhandledEventError < ::RuntimeError; end
      class PolarRuntimeException < ::RuntimeError
        def initialize(msg = '')
          super
        end
      end
      class Unimplemented < ::RuntimeError; end

      class PolarError
        attr_reader :kind, :data, :subkind
        def initialize(json)
          @kind, @data = [*json][0]
          @subkind = [*data][0]
        end
      end

      def self.get_error
        err_s = FFI.polar_get_error
        err = PolarError.new(JSON.parse(err_s))
        puts err.kind + ' Error: ' + JSON.dump(err.data)
      ensure
        FFI.string_free(err_s)
      end
    end

    class Polar
      attr_reader :pointer

      def initialize
        @pointer = FFI.polar_new
      end

      def free
        res = FFI.polar_free(pointer)
        raise FreeError if res.zero?
      end

      def load_str(str)
        res = FFI.polar_load_str(pointer, str)
        raise Error if res.zero?
      end

      def query_str(str)
        query = Query.from_str(str, pointer)
        query.results.each do |result|
          puts result
        end
      end

      def repl
        loop do
          query = Query.from_repl(pointer)
          if query.pointer.null?
            Error.get_error
            break
          end
          results = query.results.to_a
          if results.empty?
            puts 'False'
          else
            results.each do |result|
              puts result
            end
          end
        end
      end
    end

    class Query
      def self.from_str(query_str, polar)
        res = FFI.polar_new_query(polar, query_str)
        raise Error if res.null?

        new(res, polar)
      end

      def self.from_repl(polar)
        pointer = FFI.polar_query_from_repl(polar)
        new(pointer, polar)
      end

      attr_reader :pointer, :polar, :fiber

      def initialize(pointer, polar)
        @pointer = pointer
        @polar = polar
        start
      end

      def results
        Enumerator.new do |yielder|
          loop do
            result = fiber.resume
            break if result.nil?

            yielder << result
          end
        end
      end

      private

      def free
        res = FFI.query_free(pointer)
        raise FreeError if res.zero?
      end

      def start
        @fiber = Fiber.new do
          begin
            loop do
              string = FFI.polar_query(polar, pointer)
              event = JSON.parse(string)
              break if event == 'Done'

              event = Event.new(event)
              case event.kind
              when 'Result'
                Fiber.yield event.bindings
              else
                p event
                raise UnhandledEventError, event.kind
              end
            end
          ensure
            free
          end
        end
      end
    end

    class Binding
      attr_reader :var, :value

      def initialize(var, value)
        @var = var
        @value = value
      end

      def to_s
        "#{var} => #{Term.from_json(value)}"
      end
    end

    class Event
      attr_reader :kind, :data

      def initialize(json)
        @kind, @data = [*json][0]
      end

      def bindings
        data['bindings'].sort.map { |k, v| Binding.new(k, v) }
      end
    end

    class Term
      def self.from_json(json)
        tag, value = [*json['value']][0]
        case tag
        when 'Integer', 'String', 'Boolean'
          value
        when 'List'
          value.map { |term| Term.from_json(term) }
        when 'Dictionary'
          value['fields'].map { |k, v| [k, Term.from_json(v)] }.to_h
        when 'ExternalInstance', 'InstanceLiteral', 'Call'
          raise Unimplemented
        when 'Symbol'
          raise PolarRuntimeException
        else
          raise Unimplemented
        end
      end
    end
  end
end

Osohq::Polar::Polar.new.tap do |polar|
  polar.load_str('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  # polar.query_str('k(x)')

  polar.load_str('foo(1, 2); foo(3, 4); foo(5, 6);')
  # polar.query_str('foo(x, y)')

  polar.repl
end
