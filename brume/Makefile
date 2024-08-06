master:
	go run cmd/brume/main.go master

login:
	go run cmd/brume/main.go login

build-cli:
	go build -o bin/brume cmd/brume/main.go

public-gql:
	(cd ./internal/router/public-gql; go run github.com/99designs/gqlgen)