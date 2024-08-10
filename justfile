set positional-arguments

env_name := "malpractice_env"

default: hello_game

env *ARGS:
	mkdir -p ./target/fetch/git
	mkdir -p ./target/fetch/registry
	podman build -t {{env_name}} .
	podman run -v .:/workspaces/malpractice:z --rm -it {{env_name}} just {{ARGS}}

build PROFILE="debug":
    cargo build --profile {{replace_regex(PROFILE, "^debug$", "dev")}} --package hello_game

hello_game PROFILE="debug": (build PROFILE)
	./target/{{PROFILE}}/hello_game