require "dotenv"

Dotenv.load

require "sinatra"
require_relative "notification_worker"

class App < Sinatra::Application
  post "/gitlab" do
    token = request.get_header("HTTP_X_GITLAB_TOKEN")
    event = request.fetch_header("HTTP_X_GITLAB_EVENT")
    NotificationWorker.perform_async(event, request.body.read, token)
  end
end
