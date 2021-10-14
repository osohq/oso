# frozen_string_literal: true

require 'oso'

require_relative 'app'

RSpec.describe 'policy' do
  let(:alpha_association) { Organization.new 'Alpha Association' }
  let(:beta_business) { Organization.new 'Beta Business' }

  let(:affine_types) { Repository.new 'Affine Types', alpha_association }
  let(:allocator) { Repository.new 'Allocator', alpha_association }
  let(:bubble_sort) { Repository.new 'Bubble Sort', beta_business }
  let(:benchmarks) { Repository.new 'Benchmarks', beta_business }

  let(:ariana) { User.new('Ariana').tap do |a|
    a.assign_role_for_resource('owner', alpha_association)
  end }
  let(:bhavik) { User.new('Bhavik').tap do |b|
    b.assign_role_for_resource('contributor', bubble_sort)
    b.assign_role_for_resource('maintainer', benchmarks)
  end }

  it 'works' do
    oso = Oso.new

    oso.register_class Organization
    oso.register_class Repository
    oso.register_class User

    oso.load_files [File.join(File.dirname(__FILE__), 'main.polar')]

    expect { oso.authorize(ariana, 'read', affine_types) }.not_to raise_error
    expect { oso.authorize(ariana, 'push', affine_types) }.not_to raise_error
    expect { oso.authorize(ariana, 'read', allocator) }.not_to raise_error
    expect { oso.authorize(ariana, 'push', allocator) }.not_to raise_error
    expect { oso.authorize(ariana, 'read', bubble_sort) }.to raise_error Oso::NotFoundError
    expect { oso.authorize(ariana, 'push', bubble_sort) }.to raise_error Oso::NotFoundError
    expect { oso.authorize(ariana, 'read', benchmarks) }.to raise_error Oso::NotFoundError
    expect { oso.authorize(ariana, 'push', benchmarks) }.to raise_error Oso::NotFoundError

    expect { oso.authorize(bhavik, 'read', affine_types) }.to raise_error Oso::NotFoundError
    expect { oso.authorize(bhavik, 'push', affine_types) }.to raise_error Oso::NotFoundError
    expect { oso.authorize(bhavik, 'read', allocator) }.to raise_error Oso::NotFoundError
    expect { oso.authorize(bhavik, 'push', allocator) }.to raise_error Oso::NotFoundError
    expect { oso.authorize(bhavik, 'read', bubble_sort) }.not_to raise_error
    expect { oso.authorize(bhavik, 'push', bubble_sort) }.to raise_error Oso::ForbiddenError
    expect { oso.authorize(bhavik, 'read', benchmarks) }.not_to raise_error
    expect { oso.authorize(bhavik, 'push', benchmarks) }.not_to raise_error
  end
end
