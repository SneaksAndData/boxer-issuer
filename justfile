default:
    @just --list

up: start-kind-cluster build-deps install-integration-tests

fresh: stop up

stop:
    kind delete cluster --name kind

start-kind-cluster:
    kind create cluster --name kind --config=integration-tests/kind.yaml

build-deps:
    helm dependency build ./integration-tests/helm/setup

key := `openssl rand -base64 16 | tr -dc 'a-zA-Z0-9' | fold -w 16 | head -n 1`

install-integration-tests:
    helm upgrade --install --namespace default integration-tests integration-tests/helm/setup
