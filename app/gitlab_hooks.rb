# frozen_string_literal: true

require "json"
require_relative "objectified_hash"
require_relative "discord_hooks"

module GitlabHooks
  InvalidToken = Class.new(StandardError)
  HookError = Class.new(StandardError)
  Unsupported = Class.new(HookError)

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

  # incoming hook handlers

  # @param payload [ObjectifiedHash]
  # @raises Unsupported if no commits
  def push_hook(payload)
    raise Unsupported, "no commits, new branch?" \
      if payload.total_commits_count.zero?
    forward(__method__, payload)
  end

  # @param payload [ObjectifiedHash]
  # @raises Unsupported
  def merge_request_hook(payload)
    action = payload.object_attributes&.action
    raise Unsupported, "action #{action} not supported" \
      unless %w[open close merge].include?(action)
    forward(__method__, payload)
  end

  # @param payload [ObjectifiedHash]
  # @raises Unsupported
  def issue_hook(payload)
    action = payload.object_attributes&.action
    raise Unsupported, "action #{action} not supported" \
      unless %w[open close].include?(action)
    forward(__method__, payload)
  end

  # status: pending, running, success, failed
  # @param payload [ObjectifiedHash]
  # @raises Unsupported
  def pipeline_hook(payload)
    status = pipeline.status
    raise Unsupported, "status #{pipeline.status} not supported" \
      unless %w[success failed].include?(status)
    forward(__method__, payload)
  end

  HOOKS.each do |event|
    next if respond_to?(event)
    puts "Using default hook handler for #{event}"
    define_method(event) { |payload| forward(event, payload) }
  end

  # helper methods

  def validate_token(request)
    gitlab_token = request.get_header("HTTP_X_GITLAB_TOKEN")
    raise InvalidToken, "Token invalid" unless gitlab_token == TOKEN
  end

  def forward(event, payload)
    DiscordHooks.handle(event: event, payload: payload)
  end

  def handle(request)
    validate_token(request) if TOKEN
    gitlab_event = request.fetch_header("HTTP_X_GITLAB_EVENT")
    meth = gitlab_event.downcase.tr(" ", "_").to_sym
    return unless respond_to?(meth)
    payload = JSON.parse(request.body.read)
    public_send(meth, ObjectifiedHash.new(payload))
    nil
  rescue InvalidToken, HookError, JSON::ParserError => e
    puts e.message
    nil
  end
end
