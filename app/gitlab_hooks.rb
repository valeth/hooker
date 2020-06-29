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
    state = payload.object_attributes&.state
    # available actions: open, close, merge, update?
    action = payload.object_attributes&.action
    raise Unsupported, "action #{action} not supported" \
      unless %w[open close merge].include?(action)

    forward(__method__, payload)
  end

  # @param payload [ObjectifiedHash]
  # @raises Unsupported
  def issue_hook(payload)
    # available actions are: open, close, update
    action = payload.object_attributes&.action
    raise Unsupported, "action #{action} not supported" \
      unless %w[open close].include?(action)

    forward(__method__, payload)
  end

  # status: pending, running, success, failed
  # @param payload [ObjectifiedHash]
  # @raises Unsupported
  def pipeline_hook(payload)
    status = payload.object_attributes&.status
    raise Unsupported, "status #{status} not supported" \
      unless %w[success failed].include?(status)

    forward(__method__, payload)
  end

  HOOKS.each do |event|
    next if respond_to?(event)

    define_method(event) { |payload| forward(event, payload) }
  end

  # helper methods

  def validate_token(token)
    raise InvalidToken, "Token invalid" unless token == TOKEN
  end

  def forward(event, payload)
    DiscordHooks.handle(event: event, payload: payload)
  end

  def handle(event, payload, token)
    validate_token(token) if TOKEN
    meth = event.downcase.tr(" ", "_").to_sym
    return unless respond_to?(meth)

    public_send(meth, ObjectifiedHash.new(payload))
  rescue InvalidToken, HookError, JSON::ParserError => e
    $stderr.puts(e)
  end
end
