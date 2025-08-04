#!/usr/bin/env bash

set -e

# If USER_UID is not defined we skip everything and run as the default user (usually root)
if [[ ${USER_UID} ]]; then
    # If USER_UID set but USER_GID was not set then we set it to the value of USER_UID
    if [[ -z ${USER_GID} ]]; then
        USER_GID=$USER_UID
    fi
    # Now we have USER_UID and USER_GID
    # Check if USER_UID is 1000
    if [[ ${USER_UID} == "1000" ]]; then
        # If so we don't need to create a user because the Ubuntu continer already has uid=1000 setup
        echo "Running as default user: ubuntu"
    else
        # We need to change the uid/gid on the ubuntu user
        usermod -u $USER_UID ubuntu
        groupmod -g $USER_GID ubuntu
        echo "Changed uid:gid for user ubuntu to: ${USER_UID}:${USER_GID}"
        # Change ownership of the ubuntu user's homedir to the new uid
        chown -R ubuntu:ubuntu /home/ubuntu
    fi
    run_as_ubuntu=1
fi # USER_UID wasn't defined

# Now run the container's workload as either the current user or the ubuntu user
if [[ ${run_as_ubuntu} ]]; then
    su - ubuntu -c $1
else
    $1
fi
