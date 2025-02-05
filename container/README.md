# Libra-node container image

Run the container with a volume mount at `/root/.libra`, for example like this:

```shell
$ cd some-handy-place
$ mkdir libra-home
$ docker run -it --volume $(pwd)/libra-home:/root/.libra openlibracommunity/libra-node:latest libra-config fullnode-init
downloaded genesis block
config created at /root/.libra/fullnode.yaml
fullnode configs initialized  ·································· ✓
$ docker run -it --volume $(pwd)/libra-home:/root/.libra openlibracommunity/libra-node:latest -p 9101:9101 libra node
```
