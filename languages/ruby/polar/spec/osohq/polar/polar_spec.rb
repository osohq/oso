# frozen_string_literal: true

RSpec.describe Osohq::Polar::Polar do
  let(:polar) { subject }

  it 'works' do
    polar.load_str('f(1);')
    results = polar.query_str('f(x)')
    expect(results.next.transform_values(&:to_ruby)).to eq({ 'x' => 1 })
    expect { results.next }.to raise_error StopIteration

    results = polar.query_str('f(y)')
    expect(results.next.transform_values(&:to_ruby)).to eq({ 'y' => 1 })
    expect { results.next }.to raise_error StopIteration
  end
end
