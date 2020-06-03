# frozen_string_literal: true

require "dotenv"

Dotenv.load

require "sinatra"
require_relative "notification_worker"

class App < Sinatra::Application
  before do
    content_type "application/json"
  end

  post "/gitlab" do
    token = request.get_header("HTTP_X_GITLAB_TOKEN")
    event = request.get_header("HTTP_X_GITLAB_EVENT")
    if token && event
      payload = JSON.parse(request.body.read)
      NotificationWorker.perform_async(event, payload, token)
    else
      status 400
    end
  end

  get "*.php" do
    status 418
    body JSON.generate(error: "PHP")
  end

  # Fallback routes

  get "*" do
    status 404
  end

  post "*" do
    status 404
  end
end
