# frozen_string_literal: true

# TODO: add logger

require "rest-client"
require "active_support/core_ext/string/inflections"
require "active_support/core_ext/string/filters"
require "json"

module DiscordHooks
  MAX_TRIES = 3
  DISCORD_WEBHOOK_URL = ENV["DISCORD_WEBHOOK_URL"].freeze
  DISCORD_DESC_MAX = 2048
  DISCORD_TITLE_MAX = 256
  DISCORD_AUTHOR_MAX = 256
  DISCORD_FOOTER_MAX = 2048
  COLOR = {
    info:  0x1F78D1,
    alert: 0xFC9403,
    good:  0x1AAA55,
    bad:   0xDB3B21
  }.freeze

  class Ratelimited < StandardError
    attr_reader :duration

    def initialize(duration, message)
      @duration = duration
      super(message)
    end
  end

module_function

  # @param payload [ObjectifiedHash]
  # @return Hash
  def push_hook(payload)
    branch = payload.ref.split("/").last
    {
      author: author(payload.user_username, payload.user_avatar),
      title: title(payload, "#{payload.total_commits_count} new commits in #{branch}"),
      url: payload.project.web_url,
      description: join_commit_lines(payload.commits),
      color: COLOR[:info],
      footer: footer(payload)
    }
  end

  # @param payload [ObjectifiedHash]
  # @return Hash
  def merge_request_hook(payload)
    mr = payload.object_attributes

    embed = {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Merge request #{mr.state}: !#{mr.iid} #{mr.title}"),
      url: mr.url,
      footer: footer(payload),
      timestamp: Time.parse(mr.created_at).iso8601
    }
    embed[:description] = mr.description.truncate(DISCORD_DESC_MAX) unless %w[closed merged].include?(mr.state)
    embed[:color] =
      case mr.state
      when "closed" then COLOR[:alert]
      when "merged" then COLOR[:good]
      else COLOR[:info]
      end
    embed
  end

  # @param payload [ObjectifiedHash]
  # @return Hash
  def issue_hook(payload)
    issue = payload.object_attributes

    embed = {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Issue #{issue.state}: ##{issue.iid} #{issue.title}"),
      url: issue.url,
      footer: footer(payload),
      timestamp: Time.parse(issue.created_at).iso8601
    }
    embed[:description] = issue.description unless issue.state == "closed"
    embed[:color] =
      case issue.state
      when "closed" then COLOR[:good]
      else COLOR[:info]
      end
    embed
  end

  # @param payload [ObjectifiedHash]
  # @return Hash
  def note_hook(payload)
    comment = payload.object_attributes
    comment_type = comment.noteable_type.titleize.downcase

    {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "New comment on #{comment_type}"),
      url: comment.url,
      description: comment.note,
      color: 0xFC6D26,
      footer: footer(payload),
      timestamp: Time.parse(comment.created_at).iso8601
    }
  end

  # @param payload [ObjectifiedHash]
  # @return Hash
  def pipeline_hook(payload)
    pipeline = payload.object_attributes

    embed = {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Pipeline for #{pipeline.ref} #{pipeline.detailed_status} (#{pipeline.id})"),
      url: payload.commit.url,
      footer: footer(payload),
      timestamp: Time.parse(pipeline.created_at).iso8601
    }
    embed[:color] =
      case pipeline.status
      when "success" then COLOR[:good]
      when "failed" then COLOR[:bad]
      else COLOR[:info]
      end
    embed
  end

  def title(payload, after)
    "#{payload.project.name} - #{after}".truncate(DISCORD_TITLE_MAX)
  end

  def author(name, icon)
    { name: name.truncate(DISCORD_AUTHOR_MAX), icon_url: icon }
  end

  def footer(payload)
    { text: payload.project.path_with_namespace.truncate(DISCORD_FOOTER_MAX), icon_url: payload.project.avatar_url }
  end

  def join_commit_lines(commits)
    char_count = 0
    commits
      .map { |c| commit_line(c) }
      .take_while do |l|
        next false if char_count + l.size > DISCORD_DESC_MAX
        char_count += l.size
        true
      end
      .join("\n")
  end

  def commit_line(commit)
    "[`#{commit.id[0...8]}`](#{commit.url}) #{commit.message.lines.first.chomp} - **#{commit.author.name}**"
  end

  # @param event [Symbol]
  # @param payload [ObjectifiedHash]
  def handle(event:, payload:)
    tries = MAX_TRIES
    return unless respond_to?(event)
    discord_embed = public_send(event, payload)

    begin
      webhook_notify(embeds: [discord_embed])
    rescue Ratelimited => e
      if tries.zero?
        $stderr.puts("Failed to send to Discord after #{MAX_TRIES} tries, reason: #{e}")
        return
      end

      sleep(e.duration)
      tries -= 1
      retry
    end

    nil
  end

  # @param payload [Hash]
  def webhook_notify(payload)
    return unless payload.respond_to?(:to_json)
    RestClient.post(DISCORD_WEBHOOK_URL, payload.to_json, content_type: :json)
  rescue RestClient::ExceptionWithResponse => e
    parse_error_response(e.response.body)
  end

  # @param payload [Response]
  def parse_error_response(response_body)
    error_response = JSON.parse(response_body)

    case error_response["message"]
    when /.*rate limited.*/
      raise Ratelimited.new(error_response["retry_after"], error_response["message"])
    else
      $stderr.puts(error_response["message"])
    end
  rescue JSON::ParserError
    $stderr.puts(response_body)
  end
end
