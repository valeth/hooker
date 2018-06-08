require "rest-client"

module DiscordHooks
  DISCORD_WEBHOOK_URL = ENV.fetch("DISCORD_WEBHOOK_URL").freeze

module_function

  def push_hook(payload)
    {
      author: {
        name: payload.user_username,
        icon_url: payload.user_avatar
      },
      title: "[#{payload.project.path_with_namespace}] #{payload.total_commits_count} new commits",
      url: payload.project.web_url,
      description: payload.commits.reduce("") do |acc, commit|
        acc += "`#{commit.id[0..8]}` #{commit.message} - #{commit.author.name}\n"
      end,
      color: 0xFC6D26
    }
  end

  def merge_request_hook(payload)
    mr = payload.object_attributes
    {
      author: {
        name: payload.user.username,
        icon_url: payload.user.avatar_url
      },
      title: "[#{payload.project.path_with_namespace}] Merge request opened: !#{mr.iid} #{mr.title}",
      url: mr.url,
      description: mr.description,
      color: 0xE24329,
      timestamp: Time.parse(mr.created_at).iso8601
    }
  end

  def issue_hook(payload)
    issue = payload.object_attributes
    {
      author: {
        name: payload.user.username,
        icon_url: payload.user.avatar_url
      },
      title: "[#{payload.project.path_with_namespace}] Issue opened: ##{issue.iid} #{issue.title}",
      url: issue.url,
      description: issue.description,
      color: 0xfCA326,
      timestamp: Time.parse(issue.created_at).iso8601
    }
  end

  def pipeline_hook(payload)
    pipeline = payload.object_attributes
    status = (pipeline.status == "success") ? "succeeded" : "failed"
    {
      author: {
        name: payload.user.username,
        icon_url: payload.user.avatar_url
      },
      title: "[#{payload.project.path_with_namespace}] Pipeline #{status}: ##{pipeline.id}",
      url: payload.commit.url,
      description: "`#{payload.commit.id[0..8]}` #{payload.commit.message}",
      color: 0xE24329,
      timestamp: Time.parse(pipeline.created_at).iso8601
    }
  end

  def webhook_notify(payload)
    return unless payload.respond_to?(:to_json)
    RestClient.post(DISCORD_WEBHOOK_URL, payload.to_json, content_type: :json)
  rescue RestClient::ExceptionWithResponse => e
    puts e.response.body
  end

  def handle(request)
    return unless respond_to?(request.event)
    discord_embed = public_send(request.event, request.payload)
    webhook_notify(embeds: [discord_embed])
    nil
  end
end
