# frozen_string_literal: true

require "json"
require_relative "objectified_hash"
require_relative "discord_hooks"

module GitlabHooks
  HOOKS = %i[
    push_hook
    tag_push_hook
    issue_hook
    note_hook
    merge_request_hook
    wiki_page_hook
    pipeline_hook
    build_hook
  ].freeze

module_function

  HOOKS.each do |event|
    define_method(event) do |payload|
      DiscordHooks.handle(ObjectifiedHash.new(event: event, payload: payload))
    end
  end

  def handle(request)
    gitlab_event = request.fetch_header("HTTP_X_GITLAB_EVENT")
    meth = gitlab_event.downcase.tr(" ", "_").to_sym
    return unless respond_to?(meth)
    payload = JSON.parse(request.body.read)
    public_send(meth, ObjectifiedHash.new(payload))
    nil
  rescue JSON::ParserError
    nil
  end
end
