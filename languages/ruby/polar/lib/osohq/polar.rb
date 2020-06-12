# frozen_string_literal: true

require 'json'

require 'osohq/polar/version'
require 'osohq/polar/ffi'
require 'osohq/polar/errors'

module Osohq
  module Polar
    class Polar
      def initialize
        @pointer = FFI.polar_new
      end

      def load_str(str)
        res = FFI.polar_load_str(pointer, str)
        raise Error if res.zero?
      end

      def query_str(str)
        Query.from_str(str, pointer).results
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

      private

      attr_reader :pointer

      def free
        res = FFI.polar_free(pointer)
        raise Errors::FreeError if res.zero?
      end
    end

    class Query
      def self.from_str(query_str, polar)
        res = FFI.polar_new_query(polar, query_str)
        raise Errors::PolarError if res.null?

        new(res, polar)
      end

      def self.from_repl(polar)
        pointer = FFI.polar_query_from_repl(polar)
        new(pointer, polar)
      end

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

      attr_reader :pointer, :polar, :fiber

      def free
        res = FFI.query_free(pointer)
        raise Errors::FreeError if res.zero?
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
                raise Errors::UnhandledEventError, event.kind
              end
            end
          ensure
            free
          end
        end
      end
    end

    class Event
      attr_reader :kind

      def initialize(json)
        @kind, @data = [*json][0]
      end

      def bindings
        data['bindings'].sort.map { |k, v| [k, Term.new(v)] }.to_h
      end

      private

      attr_reader :data
    end

    class Term
      def self.to_ruby(value)
        tag, value = [*value][0]
        case tag
        when 'Integer', 'String', 'Boolean'
          value
        when 'List'
          value.map { |term| Term.to_ruby(term['value']) }
        when 'Dictionary'
          value['fields'].map { |k, v| [k, Term.to_ruby(v['value'])] }.to_h
        when 'ExternalInstance', 'InstanceLiteral', 'Call'
          raise Errors::Unimplemented
        when 'Symbol'
          raise Errors::PolarRuntimeException
        else
          raise Errors::Unimplemented
        end
      end

      def initialize(data)
        @id = data['id']
        @offset = data['offset']
        @value = data['value']
      end

      def to_ruby
        self.class.to_ruby(value)
      end

      private

      attr_reader :value
    end
  end
end

Osohq::Polar::Polar.new.tap do |polar|
  polar.load_str('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  # polar.query_str('k(x)')

  polar.load_str('foo(1, 2); foo(3, 4); foo(5, 6);')
  polar.query_str('foo(x, y)')

  # polar.repl
end
