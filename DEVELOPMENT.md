# Setting up development environment for the Boxer project

This document provides instructions for setting up a development environment for the Boxer project.
Follow the steps below to get started.

# Prerequisites

Before you begin, ensure you have the following installed on your system:

- Rust development environment (https://www.rust-lang.org/tools/install)
- Git (https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
- Kind (Kubernetes IN Docker) (https://kind.sigs.k8s.io/docs/user/quick-start/)
- Docker-compose (https://docs.docker.com/compose/install/)
- Golang development environment (https://golang.org/doc/install)
- kubectl (https://kubernetes.io/docs/tasks/tools/install-kubectl/)
- docker (https://docs.docker.com/get-docker/)

# Clone the Repository

To be able to run the project locally, you need to clone the following repositories:

- https://github.com/<YOUR_GITHUB_ACCOUNT>/boxer-issuer.git
- https://github.com/<YOUR_GITHUB_ACCOUNT>/boxer-validator-nginx.git
- https://github.com/<YOUR_GITHUB_ACCOUNT>/boxer-core.git
- https://github.com/<YOUR_GITHUB_ACCOUNT>/boxer-crd.git
- https://github.com/<YOUR_GITHUB_ACCOUNT>/terraform-provider-boxer.git

Replace `<YOUR_GITHUB_ACCOUNT>` with your actual GitHub username.

# Committing Changes

When you make changes to the code, ensure that you commit them to your forked repositories.
This helps in keeping your work organized and allows for easier collaboration.

GitHub actions will automatically build the project and run unit tests on every push in an active pull request in the
repository.

Additionally, GitHub actions checks if the code is properly formatted using `cargo fmt` and `go fmt`.
Please ensure that your code is formatted correctly before pushing changes.

# Testing the project locally

# Common steps

To be able to run the project locally you need to set up the keycloak server and create a realm with a client.
The keycloak server can be started using docker-compose in the project root directory:

```bash
docker-compose up
```

## With Kubernetes backend

1. For testing boxer-issuer and boxer-validator-nginx with the `kubernetes` backend you should use `kind` to create a
   local
   kubernetes cluster:

```bash
kind create cluster
kubectl config use-context kind-kind
```

2. To be able to run the project locally you need to install the latest version of the `boxer-crd` helm chart in the
   created cluster:

```bash
helm install boxer-crd <repo-location>/boxer-crd/.helm
```

Note that for local development we use the `devault` namespace in the kind cluster.

Boxer will get the kubeconfig from the kind cluster automatically using the `kind get kubeconfig --name kind` command.
This behavior can be changed by changing settings in the settings.toml file. **Please DO NOT commit changes to the
settings.toml file.**

To be able to test changes in the `boxer-core` repository you can uncomment the `boxer-core` value in the `Cargo.toml`
file that points to the local path of the `boxer-core` repository. In this case you need to comment the `boxer-core`
entry pointing to the git repository.

# Adding additional configurations for testing

The Boxer project aims to be easily configurable for development and testing purposes.
You can add additional entities to the Keycloak sever by modifying the `integration-tests/keycloak/keycloak.tf` file.

# Creating a pull request

When you are ready to create a pull request, push your changes to your forked repository and create a pull request.
Please ensure that you follow the [contribution guidelines](CONTRIBUTING.md).

# Additional setup for terraform provider development

When required CRDs are installed, you need to install the bootstrap resources in the cluster:

```bash
kubectl apply -f <repo-location>/terraform-provider-boxer/integration_tests/bootstrap.yaml
```