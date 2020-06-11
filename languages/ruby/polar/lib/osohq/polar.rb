# frozen_string_literal: true

require 'json'

require 'osohq/polar/version'
require 'osohq/polar/ffi'

module Osohq
  module Polar
    class Error < ::RuntimeError; end
    class FreeError < Error; end
    class UnhandledEventError < Error; end
    class PolarRuntimeException < Error; end
    class Unimplemented < Error; end

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
        query.run
      ensure
        query.free
      end

      def repl
        loop do
          query = Query.from_repl(pointer)
          break if query.pointer.null?

          bindings = query.run
          if bindings.empty?
            puts 'False'
          else
            puts bindings
          end
        end
      end
    end

    class Query
      attr_reader :pointer, :polar

      def initialize(pointer, polar)
        @pointer = pointer
        @polar = polar
      end

      def self.from_str(query_str, polar)
        res = FFI.polar_new_query(polar, query_str)
        raise Error if res.null?

        new(res, polar)
      end

      def self.from_repl(polar)
        pointer = FFI.polar_query_from_repl(polar)
        new(pointer, polar)
      end

      def free
        res = FFI.query_free(pointer)
        raise FreeError if res.zero?
      end

      def run
        loop do
          string = FFI.polar_query(polar, pointer)
          event = JSON.parse(string)
          break if event == 'Done'

          event = Event.new(event)
          case event.kind
          when 'Result'
            event.as_results
          else
            puts event.inspect
            raise UnhandledEventError, event.kind
          end
        end
      end
    end

    class Event
      attr_reader :kind, :data

      def initialize(json)
        @kind, @data = [*json][0]
      end

      def bindings
        data['bindings'].sort
      end

      def as_results
        bindings.each do |k, v|
          puts "#{k} => #{Term.from_json(v)}"
        end
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
  # polar.load_str('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  # polar.query_str('k(x)')

  polar.load_str('foo(1, 2); foo(3, 4); foo(5, 6);')
  polar.query_str('foo(x, y)')

  # polar.load_str('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  # polar.repl
end
