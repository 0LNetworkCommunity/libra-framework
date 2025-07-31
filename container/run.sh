# LIBRA_CONTAINER_MODE : validator|vfn|fullnode
# Currently only supports fullnode mode
#
# Check if this container has already been configured
libra_home=/root/.libra
file_indicating_already_configured="fullnode.yml"
if [[ ! -f ${libra_home}/${file_indicating_already_configured} ]]; then
	echo "No existing config detected, initializing as a fullnode..."
	# If not, run libra config
	libra config fullnode-init --archive-mode false
	echo "Initialized"
else
	echo "Container already configured"
fi
# Otherwise fall through to start node
# Start node
echo "Starting libra node"
libra node
