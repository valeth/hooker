# frozen_string_literal: true

require "rest-client"
require "active_support/core_ext/string/inflections"

module DiscordHooks
  DISCORD_WEBHOOK_URL = ENV["DISCORD_WEBHOOK_URL"].freeze

module_function

  # @param payload [ObjectifiedHash]
  # @return Hash
  def push_hook(payload)
    branch = payload.ref.split("/").last
    {
      author: author(payload.user_username, payload.user_avatar),
      title: title(payload, "#{payload.total_commits_count} new commits in #{branch}"),
      url: payload.project.web_url,
      description: payload.commits.map { |c| commit_line(c) }.join("\n"),
      color: 0xFC6D26,
      footer: footer(payload)
    }
  end

  # @param payload [ObjectifiedHash]
  # @return Hash
  def merge_request_hook(payload)
    mr = payload.object_attributes

    {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Merge request #{mr.state}: !#{mr.iid} #{mr.title}"),
      url: mr.url,
      description: mr.description,
      color: 0xE24329,
      footer: footer(payload),
      timestamp: Time.parse(mr.created_at).iso8601
    }
  end

  # @param payload [ObjectifiedHash]
  # @return Hash
  def issue_hook(payload)
    issue = payload.object_attributes

    embed = {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Issue #{issue.state}: ##{issue.iid} #{issue.title}"),
      url: issue.url,
      color: 0xfCA326,
      footer: footer(payload),
      timestamp: Time.parse(issue.created_at).iso8601
    }
    embed[:description] = issue.description unless issue.state == "closed"
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

    {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Pipeline for #{pipeline.ref} #{pipeline.detailed_status} (#{pipeline.id})"),
      url: payload.commit.url,
      color: 0xE24329,
      footer: footer(payload),
      timestamp: Time.parse(pipeline.created_at).iso8601
    }
  end

  def title(payload, after)
    "#{payload.project.name} - #{after}"
  end

  def author(name, icon)
    { name: name, icon_url: icon }
  end

  def footer(payload)
    { text: payload.project.path_with_namespace, icon_url: payload.project.avatar_url }
  end

  def commit_line(commit)
    "[`#{commit.id[0...8]}`](#{commit.url}) #{commit.message.lines.first.chomp} - **#{commit.author.name}**"
  end

  # @param event [Symbol]
  # @param payload [ObjectifiedHash]
  def handle(event:, payload:)
    return unless respond_to?(event)
    discord_embed = public_send(event, payload)
    webhook_notify(embeds: [discord_embed])
    nil
  end

  # @param payload [Hash]
  def webhook_notify(payload)
    return unless payload.respond_to?(:to_json)
    RestClient.post(DISCORD_WEBHOOK_URL, payload.to_json, content_type: :json)
  rescue RestClient::ExceptionWithResponse => e
    puts e.response.body
  end
end
