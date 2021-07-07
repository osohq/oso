# frozen_string_literal: true

module RolesHelpers
  class Org
    attr_reader :name

    def initialize(name)
      @name = name
    end
  end

  class Repo
    attr_reader :name, :org

    def initialize(name, org)
      @name = name
      @org = org
    end
  end

  class Issue
    attr_reader :name, :repo

    def initialize(name, repo)
      @name = name
      @repo = repo
    end
  end

  class Role
    attr_reader :name, :resource

    def initialize(name, resource)
      @name = name
      @resource = resource
    end
  end

  class User
    attr_reader :name, :roles

    def initialize(name, roles)
      @name = name
      @roles = roles
    end
  end
end
