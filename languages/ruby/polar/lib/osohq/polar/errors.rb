# frozen_string_literal: true

module Osohq
  module Polar
    # Base error type for the Osohq::Polar library.
    class Error < RuntimeError; end

    class FFIError < Error; end
    class FreeError < FFIError; end
    class PolarRuntimeException < Error; end
    class UnhandledEventError < Error; end
    class UnimplementedError < Error; end

    # Catch-all for a parsing error that doesn't match any of the more specific types.
    class ParseError < Error
      # @param kind [String]
      # @param details [Hash<String, #to_s>]
      def initialize(kind:, details:)
        super("ParseError - #{kind} - #{details}")
        @kind = kind
        @details = details
      end

      # Unexpected additional token.
      class ExtraToken < ParseError
        # @param token [String]
        # @param pos [Array<(Fixnum, Fixnum)>]
        def initialize(token:, pos:)
          super(kind: 'ExtraToken', details: { 'token' => token, 'pos' => pos })
        end
      end

      # These go to eleven.
      class IntegerOverflow < ParseError
        # @param token [String]
        # @param pos [Array<(Fixnum, Fixnum)>]
        def initialize(token:, pos:)
          super(kind: 'IntegerOverflow', details: { 'token' => token, 'pos' => pos })
        end
      end

      # TODO(gj): document
      class InvalidTokenCharacter < ParseError
        # @param token [String]
        # @param char [String]
        # @param pos [Array<(Fixnum, Fixnum)>]
        def initialize(token:, char:, pos:)
          super(kind: 'InvalidTokenCharacter', details: { 'token' => token, 'char' => char, 'pos' => pos })
        end
      end

      # TODO(gj): document
      class InvalidToken < ParseError
        # @param pos [Array<(Fixnum, Fixnum)>]
        def initialize(pos:)
          super(kind: 'InvalidToken', details: { 'pos' => pos })
        end
      end

      # TODO(gj): document
      class UnrecognizedEOF < ParseError
        # @param pos [Array<(Fixnum, Fixnum)>]
        def initialize(pos:)
          super(kind: 'UnrecognizedEOF', details: { 'pos' => pos })
        end
      end

      # TODO(gj): document
      class UnrecognizedToken < ParseError
        # @param token [String]
        # @param pos [Array<(Fixnum, Fixnum)>]
        def initialize(token:, pos:)
          super(kind: 'UnrecognizedToken', details: { 'token' => token, 'pos' => pos })
        end
      end
    end
  end
end
