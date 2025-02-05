# Libra-node container image

Run the container with a volume mount at `/root/.libra`, for example like this:

```shell
$ cd some-handy-place
$ mkdir libra-home
$ export libra_container_tag=ghcr.io/0lnetworkcommunity/libra-framework/libra-node:latest
$ docker run -it --volume $(pwd)/libra-home:/root/.libra ${libra_container_tag} libra-config fullnode-init
downloaded genesis block
config created at /root/.libra/fullnode.yaml
fullnode configs initialized  ·································· ✓
$ docker run -it -d --volume $(pwd)/libra-home:/root/.libra -p 9101:9101 -p 6182:6182 -p 8080:8080 ${libra_container_tag} libra node
```
