# frozen_string_literal: true

require_relative './helpers'
require 'osohq/polar/errors'

RSpec.configure do |c|
  c.include Helpers
end

RSpec.describe Osohq::Polar::Polar do
  let(:test_file) { File.join(__dir__, 'test_file.polar') }
  let(:test_file_gx) { File.join(__dir__, 'test_file_gx.polar') }

  it 'works' do
    subject.load_str('f(1);')
    results = subject.query_str('f(x)')
    expect(results.next).to eq({ 'x' => 1 })
    expect { results.next }.to raise_error StopIteration
  end

  it 'converts Polar values into Ruby values' do
    subject.load_str('f({x: [1, "two", true], y: {z: false}});')
    expect(qvar(subject, 'f(x)', 'x', one: true)).to eq({ 'x' => [1, 'two', true], 'y' => { 'z' => false } })
  end

  context '#load' do
    before(:example) { pending 'Polar#load is unimplemented' }

    it 'loads a Polar file' do
      subject.load(test_file)
      expect(qvar(subject, 'f(x)', 'x')).to eq([1, 2, 3])
    end

    it 'raises if given a non-Polar file' do
      expect { subject.load('other.ext') }.to raise_error Errors::BadFile
    end

    it 'is idempotent' do
      2.times { subject.load(test_file) }
      expect(qvar(subject, 'f(x)', 'x')).to eq([1, 2, 3])
    end

    it 'can load multiple files' do
      subject.load(test_file)
      subject.load(test_file_gx)
      expect(qvar(subject, 'f(x)', 'x')).to eq([1, 2, 3])
      expect(qvar(subject, 'g(x)', 'x')).to eq([1, 2, 3])
    end
  end

  context '#clear' do
    before(:example) { pending 'Polar#clear is unimplemented' }

    it 'clears the KB' do
      subject.load(test_file)
      subject.clear
      expect(query(subject, 'f(x)')).to be false
    end
  end
end
