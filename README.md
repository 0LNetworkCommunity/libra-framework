# LIBRA FRAMEWORK

The Libra Framework is a Move framework that can run on Diem Platform nodes. It contains the policies for the 0L network. [See here a brief history of this source](./docs/core_devs/0_engineering_strategy.md#commit-history).

## What are Move Frameworks
Move frameworks are source code written in the Move Language. In Diem Platform and their vendor versions, the framework does all most all of the state machine's business logic: account creation, coin minting, asset transferring, code publishing, code upgrades, selecting validators, etc.

## Relevance
For the 0L network, which intends to be agnostic about platform vendors, but very opinionated about policies and economics, the majority of the community's innovations are found here.

# What's included
There are many tools needed for the framework to be properly developed, installed, upgraded, transacted with, monitored, etc.

# What's not included
There isn't any code regarding database, consensus, networking, virtual machines here. That code belongs in Diem Platform. There are many vendors of the Diem technologies, and 0L maintains a version which brings in maintenance upgrades from our peers.

# Technology Strategy
In general, we have a light touch approach with our upstream dependencies, and optimize for clean APIs easily pull changes from vendors (and even ultimately have freedom to change infrastructure vendors).

[Read Engineering Strategy](./docs/core_devs/0_engineering_strategy.md)

### [Go To Documentation](./docs/README.md)

Smart Contract Devs should [start here](./docs/publishing_smart_contracts.md).

Core Devs should [start here](./docs/core_devs/dev_quick_start.md).