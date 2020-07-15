# frozen_string_literal: true

require 'oso'

OSO ||= Oso.new
OSO.allow(actor: 'alice', action: 'approve', resource: 'expense')
