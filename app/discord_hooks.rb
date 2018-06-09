require "rest-client"
require "active_support/core_ext/string/inflections"

module DiscordHooks
  Unsupported = Class.new(StandardError)

  DISCORD_WEBHOOK_URL = ENV.fetch("DISCORD_WEBHOOK_URL").freeze

module_function

  def push_hook(payload)
    {
      title: title(payload, "#{payload.total_commits_count} new commits"),
      url: payload.project.web_url,
      description: payload.commits.map { |c| commit_line(c) }.join("\n"),
      color: 0xFC6D26,
      footer: footer(payload)
    }
  end

  def merge_request_hook(payload)
    mr = payload.object_attributes
    status =
      case mr.action
      when "open" then "opened"
      when "close" then "closed"
      else raise Unsupported, "action #{mr.action} not supported"
      end

    {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Merge request #{status}: !#{mr.iid} #{mr.title}"),
      url: mr.url,
      description: mr.description,
      color: 0xE24329,
      footer: footer(payload),
      timestamp: Time.parse(mr.created_at).iso8601
    }
  end

  def issue_hook(payload)
    issue = payload.object_attributes
    status =
      case issue.action
      when "open" then "opened"
      when "close" then "closed"
      else raise Unsupported, "action #{issue.action} not supported"
      end

    {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Issue #{status}: ##{issue.iid} #{issue.title}"),
      url: issue.url,
      description: issue.description,
      color: 0xfCA326,
      footer: footer(payload),
      timestamp: Time.parse(issue.created_at).iso8601
    }
  end

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

  def pipeline_hook(payload)
    pipeline = payload.object_attributes
    return unless %w[success failed].include?(pipeline.status)

    status =
      case pipeline.status
      when "success" then "succeeded"
      when "failed"  then "failed"
      else raise Unsupported, "status #{pipeline.status} not supported"
      end

    {
      author: author(payload.user.username, payload.user.avatar_url),
      title: title(payload, "Pipeline #{status}: ##{pipeline.id}"),
      url: payload.commit.url,
      description: commit_line(payload.commit),
      color: 0xE24329,
      footer: footer(payload),
      timestamp: Time.parse(pipeline.created_at).iso8601
    }
  end

  def title(payload, after)
    "#{payload.project.name} - #{after}"
  end

  def author(name, icon)
    { name: name, icon: icon }
  end

  def footer(payload)
    { text: payload.project.path_with_namespace, icon_url: payload.project.avatar_url }
  end

  def commit_line(commit)
    "[`#{commit.id[0..8]}`](#{commit.url}) #{commit.message.lines.first.chomp} - **#{commit.author.name}**"
  end

  def handle(request)
    return unless respond_to?(request.event)
    discord_embed = public_send(request.event, request.payload)
    webhook_notify(embeds: [discord_embed])
    nil
  rescue Unsupported
    nil
  end

  def webhook_notify(payload)
    return unless payload.respond_to?(:to_json)
    RestClient.post(DISCORD_WEBHOOK_URL, payload.to_json, content_type: :json)
  rescue RestClient::ExceptionWithResponse => e
    puts e.response.body
  end
end
