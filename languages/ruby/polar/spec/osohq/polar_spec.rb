# frozen_string_literal: true

RSpec.describe Osohq::Polar do
  it 'has a version number' do
    expect(Osohq::Polar::VERSION).not_to be nil
  end

  it 'does something useful' do
    expect(false).to eq(true)
  end

  it 'works' do
    p = Osohq::Polar::Polar.new
    p.load_str('f(1);')
    p.query_str('f(x)')

    results = list(p._query_str('f(x)'))
    assert results[0]['x'] == 1
    results = list(p._query_str('f(y)'))
    assert results[0]['y'] == 1
    del p
  end
end
