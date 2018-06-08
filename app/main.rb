require "dotenv"

Dotenv.load

require "sinatra"
require_relative "gitlab_hooks"

class App < Sinatra::Application
  post "/gitlab" do
    GitlabHooks.handle(request)
  end
end
