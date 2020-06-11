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
        def initialize(msg="")
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
        err_s = FFI.polar_get_error()
        err = PolarError.new(JSON.parse(err_s))
        puts err.kind + " Error: " + JSON.dump(err.data)

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
        results = query.run
        if results.empty?
          puts 'False'
        else
          results.each {|res| Event.print_result(res)}
        end
      ensure
        query.free
      end

      def repl
        had_result = false
        loop do
          query = Query.from_repl(pointer)
          if query.pointer.null?
            Error::get_error()
            break
          end
          results = query.run
          if results.empty?
            puts 'False'
          else
            results.each {|res| Event.print_result(res)}
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

      # run a query
      def run
        # hack around not having generators
        results = Array.new
        loop do
          string = FFI.polar_query(polar, pointer)
          event = JSON.parse(string)
          if event == 'Done'
            break
          end
          event = Event.new(event)
          case event.kind
          when 'Result'
            results.push(event.data['bindings'])
          else
            puts event.inspect
            raise UnhandledEventError, event.kind
          end
        end
        return results
      end
    end

    class Event
      attr_reader :kind, :data

      def initialize(json)
        @kind, @data = [*json][0]
      end

      def self.print_result(bindings)
        if bindings.empty?
          puts "True"
        else
          bindings.each do |k, v|
            puts "#{k} => #{Term.from_json(v)}"
          end
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
  polar.repl

  # polar.load_str('f(1); f(2); g(1); g(2); h(2); k(x) := f(x), h(x), g(x);')
  # polar.repl
end
