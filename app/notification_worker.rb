require "sidekiq"
require_relative "gitlab_hooks"

class NotificationWorker
  include Sidekiq::Worker

  def perform(event, payload, token)
    GitlabHooks.handle(event, payload, token)
  end
end
