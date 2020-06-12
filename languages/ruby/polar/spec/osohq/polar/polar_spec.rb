# frozen_string_literal: true

require_relative './helpers'

RSpec.configure do |c|
  c.include Helpers
end

RSpec.describe Osohq::Polar::Polar do
  it 'works' do
    subject.load_str('f(1);')
    results = subject.query_str('f(x)')
    expect(results.next).to eq({ 'x' => 1 })
    expect { results.next }.to raise_error StopIteration
  end

  it 'converts Polar values into Ruby values' do
    subject.load_str('f({x: [1, "two", true], y: {z: false}});')
    expect(qvar(subject, 'f(x)', 'x')).to eq({ 'x' => [1, 'two', true], 'y' => { 'z' => false } })
  end
end
