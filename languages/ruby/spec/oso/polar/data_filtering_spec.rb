# frozen_string_literal: true

require 'tempfile'

require_relative './helpers'

RSpec.configure do |c|
  c.include Helpers
end

RSpec.describe Oso::Polar::Polar do # rubocop:disable Metrics/BlockLength
  let(:test_file) { File.join(__dir__, 'test_file.polar') }
  let(:test_file_gx) { File.join(__dir__, 'test_file_gx.polar') }

  context 'data filtering' do
    context 'when filtering known values' do
      it 'works' do
        subject.load_str('allow(_, _, i) if i in [1, 2];')
        subject.load_str('allow(_, _, i) if i = {};')

        expect(subject.get_allowed_resources('gwen', 'get', Integer)).to eq([1,2])
        expect(subject.get_allowed_resources('gwen', 'get', Hash)).to eq([{}])
      end
    end
  end

end
