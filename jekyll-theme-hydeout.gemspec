# coding: utf-8

Gem::Specification.new do |spec|
  spec.name          = "jekyll-theme-hydeout"
  spec.version       = "0.1.0"
  spec.authors       = ["Stefan Lankes"]
  spec.email         = ["slankes@eonerc.rwth-aachen.de"]

  spec.summary       = %q{A teaching operating system written in Rust}
  spec.homepage      = "https://rwth-os.github.io/eduOS-rs/"
  spec.license       = "MIT"

  spec.metadata["plugin_type"] = "theme"

  spec.files         = `git ls-files -z`.split("\x0").select do |f|
    f.match(%r{^(assets|_(includes|layouts|sass)/|(LICENSE|README)((\.(txt|md|markdown)|$)))}i)
  end

  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }

  spec.add_runtime_dependency "jekyll", "~> 3.4"
  spec.add_runtime_dependency "jekyll-gist", "~> 1.4"
  spec.add_runtime_dependency "jekyll-paginate", "~> 1.1"
  spec.add_runtime_dependency "jekyll-feed", "~> 0.6"
  spec.add_development_dependency "bundler", "~> 1.12"
end
