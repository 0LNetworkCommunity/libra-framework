#!/bin/bash

# Extract the value from validator_network_public_key
validator_key=$(sed -n 's/^validator_network_public_key: "\(.*\)"/\1/p' ~/.libra/operator.yaml)

# Use the extracted value to replace full_node_network_public_key
sed -i "s/^full_node_network_public_key: .*/full_node_network_public_key: \"$validator_key\"/" ~/.libra/operator.yaml

# File path
file="$HOME/.libra/operator.yaml"

# Extract the host and port from validator_host
host=$(awk '/^  host:/{print $0}' "$file")
port=$(awk '/^  port:/{print $0}' "$file")

# Replace full_node_host: ~ with full_node_host: and add host and port
sed -i "/^full_node_host: ~/c\\
full_node_host:\\n$host\\n$port" "$file"
