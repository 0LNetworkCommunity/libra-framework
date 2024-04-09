#!/bin/bash

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

nc -k -l -p 6180 &
nc -k -l -p 6181 &
nc -k -l -p 6182 &
nc -k -l -p 8080 &

echo "Waiting forever, hit ctrl-c to exit cleanly"

sleep infinity
