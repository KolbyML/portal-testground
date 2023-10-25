#! /bin/bash

# Immediately abort the script on any error encountered
set -e

# 1.
if [[ "$client_type" == 'trin' ]]; then
  registry_type='1'
  image="portalnetwork/trin"
  version="latest"
elif [[ "$client_type" == 'fluffy' ]]; then
  registry_type='1'
  image="statusim/nimbus-fluffy"
  version="amd64-master-latest"
elif [[ "$client_type" == 'ultralight' ]]; then
  registry_type='2'
  image="ethereumjs/ultralight"
  version="latest"
else
  echo "client_type wasn't set, set it by `export client_type=trin`"
  exit 1
fi
folder_filter=""

#echo "nameserver 0.0.0.0" >> /etc/resolv.conf
#resolvconf -u

# are we using docker hub or github containers
# 1. docker hub
# 2. github containers
docker_hub_auth="https://auth.docker.io/token?service=registry.docker.io&scope=repository:"
docker_hub_api="https://registry-1.docker.io/v2/"
github_containers_auth="https://ghcr.io/token?scope=repository:"
github_containers_api="https://ghcr.io/v2/"

if [[ "$registry_type" == '1' ]]; then
  auth_url=$docker_hub_auth
  api_url=$docker_hub_api
elif [[ "$registry_type" == '2' ]]; then
  auth_url=$github_containers_auth
  api_url=$github_containers_api
else
  echo "This should be impossible $registry_type"
  exit 1
fi

# 2.
work_dir=.
target_dir="/"

# https://github.com/distribution/distribution/blob/main/docs/spec/manifest-v2-2.md
docker_manifest_v2="application/vnd.docker.distribution.manifest.v2+json"
# https://github.com/opencontainers/image-spec/blob/main/manifest.md
oci_manifest_v1="application/vnd.oci.image.manifest.v1+json"
# Docker Hub can return either type of manifest format. Most images seem to
# use the Docker format for now, but the OCI format will likely become more
# common as features that require that format become enabled by default
# (e.g., https://github.com/docker/build-push-action/releases/tag/v3.3.0).
accept_header="Accept: ${oci_manifest_v1}"

# 3.
cd "$work_dir"

# 4. get an API token
echo "Getting an API token"
token=$(curl --silent --header 'GET' "$auth_url$image:pull" | jq -r '.token')

# 5. download manifest to get layers
echo "Retrieving $image:$version layers list"
if [[ "$registry_type" == '1' ]]; then
  layers=$(curl --silent --request 'GET' --header "$accept_header" --header "Authorization: Bearer $token" "$api_url$image/manifests/$version" | jq -r '(reduce [.fsLayers[].blobSum][] as $a ([]; if IN(.[]; $a) then . else . += [$a] end))[]')
elif [[ "$registry_type" == '2' ]]; then
  layers=$(curl --silent --request 'GET' --header "$accept_header" --header "Authorization: Bearer $token" "$api_url$image/manifests/$version" | jq -r '(reduce [.layers[].digest][] as $a ([]; if IN(.[]; $a) then . else . += [$a] end))[]')
fi

echo $layers
# we want to filter out the first layer which is the base image
if [[ "$client_type" == 'ultralight' ]]; then
  # ultralights structure is weird so we will just take it all
  layer_filter=""
  layers=$(echo -e "$layers" | tail --lines 1 | head --lines 1)
else
  layer_filter=$(echo -e "$layers" | tail --lines 1 | head --lines 1)
fi
layer_filter=${layer_filter/*:/}

# 6. download and extract each layer
mkdir -p "layers/gz"
mkdir -p "layers/tar"
for i in $layers; do
  name="${i/*:/}"
  if [[ "$name" == "$layer_filter" ]]; then
    continue
  fi
  out="layers/gz/$name.gz"
  echo "Downloading layer $name"
  curl --silent --location --request 'GET' --header "Authorization: Bearer $token" "$api_url$image/blobs/$i" > "$out"
  gunzip -c "$out" > "layers/tar/$name"
  rm "$out"
done

# 7. for each layer extract the actual files in the target directory
mkdir -p "$target_dir"
for i in layers/tar/*; do
  if tar -tf "$i" "$folder_filter" >/dev/null 2>&1; then
    echo "Extracting $i"
    tar -xf "$i" -C "$target_dir" "$folder_filter"
  else
    echo "No $folder_filter in $i, skipping"
  fi
done
rm -rf "layers"
echo "Created $target_dir"