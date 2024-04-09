#!/bin/bash

# Prompt user to define node type
read -p "Will this node run as a Validator (VN) or VFN? Enter 'VN' or 'VFN': " node_type

# Cleanup existing UFW rules related to the application
echo "Resetting UFW rules for ports 8080, 6180, 6181, and 6182..."
ufw delete allow 8080;
ufw delete allow 6180;
ufw delete allow 6181;
ufw delete allow 6182;

if [ "$node_type" == "VN" ]; then
    # Configure for Validator Node (VN)
    echo "Configuring as Validator Node..."
    read -p "Provide your VFN IP otherwise use 127.0.0.1 in its place: " vfn_ip
    vfn_ip="${vfn_ip:-127.0.0.1}" # Default IP if none provided
    ufw allow from $vfn_ip to any port 6181
    ufw allow 6180
elif [ "$node_type" == "VFN" ]; then
    # Configure for Full Node (VFN)
    echo "Configuring as Full Node..."
    read -p "Provide your Validator IP otherwise use 127.0.0.1 in its place: " val_ip
    val_ip="${val_ip:-127.0.0.1}" # Default IP if none provided
    ufw allow from $val_ip to any port 6181
    ufw allow 8080
    ufw allow 6182
else
    echo "Invalid node type entered. Please run the script again and enter 'VN' or 'VFN'."
    exit 1
fi

echo "UFW configuration completed successfully."
