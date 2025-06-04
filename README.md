# LIBRA FRAMEWORK
[![rust-versions](https://github.com/0LNetworkCommunity/libra-framework/actions/workflows/rust-versions.yaml/badge.svg)](https://github.com/0LNetworkCommunity/libra-framework/actions/workflows/rust-versions.yaml) [![rust ci](https://github.com/0LNetworkCommunity/libra-framework/actions/workflows/ci.yaml/badge.svg)](https://github.com/0LNetworkCommunity/libra-framework/actions/workflows/ci.yaml) [![move framework tests](https://github.com/0LNetworkCommunity/libra-framework/actions/workflows/move.yaml/badge.svg)](https://github.com/0LNetworkCommunity/libra-framework/actions/workflows/move.yaml)

## OpenLibra
The Libra Framework is a Move language framework that can run on Diem Platform nodes. It
contains the policies for the $LBR asset and the OpenLibra Network.

For the OpenLibra network member (who are very opinionated about policies and
economics) the majority of the community's innovations are found here.

## What are Move Frameworks

Move frameworks are source code written in the Move Language. In Diem Platform
(and in vendor versions), the framework contains almost all of the platform's
business logic: account creation, coin minting, asset transferring, code
publishing, code upgrades, selecting validators, etc.


# What's Included

There are many tools needed for the framework to be properly developed, tested, installed, upgraded, transacted with, monitored, etc.

# What's Not Included

There isn't any code regarding database, consensus, networking, virtual
machines, etc. here. That code belongs in Diem Platform. There are many vendors of
the Diem technologies, and OpenLibra maintains a version with linear commits
back to Facebook's version, and updates it with maintenance
upgrades from contemporary vendors.

# Technology Strategy
In general, we have a light touch approach with our upstream dependencies. We
optimize for clean APIs that easily pull in libraries from vendors. Ultimately,
the OpenLibra code devs are agnostic about blockchain technology, so long as it
has the necessary components for the mission of $LBR.


Core Devs should [start here](https://docs.openlibra.io/guides/developers/dev-quick-start).
