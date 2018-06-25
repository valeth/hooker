worker_processes ENV.fetch("UNICORN_WORKERS") { `nproc`&.to_i || 4 }

working_directory File.expand_path(".", __dir__)

preload_app true

timeout 30

listen "0.0.0.0:9292"

initialized = false

before_fork do |server, worker|
  next if initialized

  Thread.new do
    require "childprocess"
    ChildProcess.build("sidekiq", "-r", "./app/main.rb").tap do |p|
      p.io.inherit!
      p.start
      p.wait
    end
  rescue => e
    warn e.message
    warn e.backtrace.join("\n")
    exit 1
  end

  initialized = true
end
