worker_processes `nproc`&.to_i || 4

working_directory File.expand_path(".", __dir__)

timeout 30

listen "0.0.0.0:9292"
