default:
    @just --list

# Updates the helm dependencies for the integration tests. This should be run after any changes to the helm charts in
# integration-tests/helm/setup. This receipe updates the Chart.lock file that should be committed to the repo.

update-deps:
    helm dependency update ./integration-tests/helm/setup

up: start-kind-cluster build-deps integration-tests keycloak ingress-controller wait-for-services ingress token-secret

fresh: stop up

stop:
    kind delete cluster --name kind

start-kind-cluster:
    kind create cluster --name kind --config=integration-tests/kind.yaml

build-deps:
    helm dependency build ./integration-tests/helm/setup

key := `openssl rand -base64 16 | tr -dc 'a-zA-Z0-9' | fold -w 16 | head -n 1`

integration-tests:
    helm upgrade --install --namespace default integration-tests integration-tests/helm/setup \
      --set-literal 'boxer-validator-nginx.validator.config.tokenSettings.keys={"default": "{{ key }}"}' \
      --set 'boxer-validator-nginx.validator.config.listenIp=0.0.0.0' \
      --set 'boxer-validator-nginx.validator.config.backend.kubernetes.resourceOwnerLabel=application/boxer-validator-nginx' \
      --set boxer-validator-nginx.validator.replicas=1

keycloak:
    helm upgrade --install keycloak oci://ghcr.io/codecentric/helm-charts/keycloakx \
      --set keycloak.username=admin \
      --set keycloak.password=admin \
      --values ./integration_tests/keycloak.yaml

ingress-controller:
    kubectl apply -f https://kind.sigs.k8s.io/examples/ingress/deploy-ingress-nginx.yaml

wait-for-services:
    kubectl rollout status deployment/boxer-validator-nginx --timeout=180s
    kubectl rollout status deployment/ingress-nginx-controller --namespace ingress-nginx --timeout=180s
    kubectl rollout status statefulset/keycloak-keycloakx --timeout=180s

ingress:
    # Wait a bit for ingress controller to be ready to accept rules
    sleep 10
    # Create ingress rules for boxer-issuer and boxer-validator-nginx
    kubectl apply -f ./integration_tests/ingress.yaml

token-secret:
    kubectl create secret generic boxer-issuer-token-settings --from-literal=BOXER__TOKEN_SETTINGS__KEY='{{ key }}'
