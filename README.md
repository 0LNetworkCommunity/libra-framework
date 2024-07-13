# LIBRA FRAMEWORK

The Libra Framework is a Move language framework that can run on Diem Platform nodes. It
contains the policies for the $LIBRA asset and the Open Libra Network.

For the Open Libra network members, who are very opinionated about policies and
economics, the majority of the community's innovations are found here.

## What are Move Frameworks

Move frameworks are source code written in the Move Language. In Diem Platform
(and in vendor versions), the framework contains almost all of the platform's
business logic: account creation, coin minting, asset transferring, code
publishing, code upgrades, selecting validators, etc.


# What's Included

There are many tools needed for the framework to be properly developed, installed, upgraded, transacted with, monitored, etc.

# What's Not Included

There isn't any code regarding database, consensus, networking, virtual
machines, etc. here. That code belongs in Diem Platform. There are many vendors of
the Diem technologies, and Open Libra maintains a version with linear commits
back to Facebook's version, and updates it with maintenance
upgrades from contemporary vendors.

# Technology Strategy
In general, we have a light touch approach with our upstream dependencies. We
optimize for clean APIs that easily pull in libraries from vendors. Ultimately,
the Open Libra members are agnostic about blockchain technology, so long as it
has the necessary components for the mission of $LIBRA.

[Read details here: ](https://0lnetwork.dev/about/0_engineering_strategys)


Core Devs should [start here](https://0lnetwork.dev/developers/getting-started).
