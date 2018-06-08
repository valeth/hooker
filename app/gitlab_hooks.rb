# frozen_string_literal: true

require "json"
require_relative "objectified_hash"
require_relative "discord_hooks"

module GitlabHooks
  InvalidToken = Class.new(StandardError)

  TOKEN = ENV["GITLAB_TOKEN"].freeze
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

  def validate_token(request)
    gitlab_token = request.get_header("HTTP_X_GITLAB_TOKEN")
    raise InvalidToken, "Token invalid" unless gitlab_token == TOKEN
  end

  def handle(request)
    validate_token(request) if TOKEN
    gitlab_event = request.fetch_header("HTTP_X_GITLAB_EVENT")
    meth = gitlab_event.downcase.tr(" ", "_").to_sym
    return unless respond_to?(meth)
    payload = JSON.parse(request.body.read)
    public_send(meth, ObjectifiedHash.new(payload))
    nil
  rescue InvalidToken, JSON::ParserError => e
    puts e.message
    nil
  end
end
