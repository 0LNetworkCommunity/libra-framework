# ENGINEERING STRATEGY

The engineering strategy is in line with the overall community goal: optimize for being opinionated about economic policy, and having long-term optionality in picking infrastructure vendors.

In 0L the content (our coin) is the priority. It's too risky being your own infrastructure stack vendor during a standards war. Instead, optimize for longevity and portability of the native asset and its policies.

As such, 0L's innovations are in the middleware and application layers primarily in regards to economic policy. We intentionally rely heavily on (wonderful and cutting edge) infrastructure vendors, and aim to be agnostic for when the inevitable next technology wave arrives.

# DEPENDENCY MANAGEMENT

Our code management strategy means we separate some concerns and have clear APIs between each major system component.

- `Libra Framework`: is responsible for the framework source and tools to test and deploy etc. This is mostly original source that wraps or relies heavily on Move Language and Diem Platform (see below).
- `Carpe`: is currently a wallet. Before version V7, it was also a light miner that included the tower. This is original source that imports our libra-tools.
- `Tower`: is the verifiable delay function proof-of-work system 0L used to bootstrap and mint all the coins up to V7. This is original source, lightly uses `libra-tools`.
- `Libra-Tools`: low level CLIs and libraries for wallets, transactions, queries. This is original source that relies heavily on Diem Platform.
- `Diem Platform`: is infrastructural network node source (database, consensus, networking, configs, testing). There are linear commits to Facebook's Diem and today is maintained primarily by Matonee Inc (Aptos).
- `Move Language`: is the language spec, standard library, virtual machine, and dev tools. There are linear commits to Facebook's Move source, and today is primarily maintained by Mysten Labs Inc (Sui).

# Commit History

## Hello World

The commit history of `ol/diem-platform` will show that 0L forked Facebook's Libra (later renamed Diem) on their launch day in June 2019. Facebook was our first vendor.

## 2019

A majority of the work in 2019 existed in separated now deprecated projects; `ol/movemint` for example, based on Cosmos SDK tooling.

On the `diem-platform` repo the majority of the work began in Q4 2019, where we maintained the Diem monorepo structure.

We added our cryptography ceremony (Towers) and policy layers to mostly \*.mvir (Yes, we were writing in the Move Intermediary Representation before the Move language existed properly!).

## 2020

The V1 0L public mainnet was launched in Q3 2020 <!--(TODO: Links to commits)-->. Fun fact, you can see our crypto seeds based on contemporaneous news of the pandemic period.

That code was based on commit <!--(TODO: XYZ)--> from Facebook Diem v 1.4.

## 2021

The V2 Public Network happened in November 2021, based on Facebook Diem 1.5 <!--(TODO: Commit)-->.

Mysten Lab's Sui platform started their Move framework on Dec 2 2021: https://github.com/MystenLabs/sui/commit/2c26b167a3fede5c892a64a4d6c68e6b56ed2e30.

## 2022

The Move Language repo became decoupled into its own repo on January 7th 2022. See `move-language` [first commit here](https://github.com/move-language/move/commits/98ed299a7e3a9223019c9bdf4dd92fea9faef860). Compare: [diem#5af34](https://github.com/0LNetworkCommunity/diem/commit/5af341e53c9a24ebe24596f4890fcaaef3bcdc54), and [move-language#5af34](https://github.com/move-language/move/commit/5af341e53c9a24ebe24596f4890fcaaef3bcdc54).

On Feb 18th 2022, Facebook effectively ended maintenance of Diem at commit `50925260829a84d7a07b7f1922fcefc3b8147a2d`.

0L V6 was based on that end-of-the-line Diem code.

Matonee's Aptos Platform forked Diem that commit. Compare: [aptos#50925](https://github.com/aptos-labs/aptos-core/commit/50925260829a84d7a07b7f1922fcefc3b8147a2d) and [diem#50925](https://github.com/diem/diem/commit/50925260829a84d7a07b7f1922fcefc3b8147a2d). More info here: https://github.com/aptos-labs/aptos-core/pull/14

## 2023

We split our `libra` repo (with linear history to Facebook's first commit) into `libra-framework` and `diem-platform`. . As far as we know this is the only maintained version of Libra aside from vendor forks.

0L continues to maintain Diem under `diem-platform` pulling in the (excellent and ninja) maintenance upgrades from Matonee since commit `#509252`. V7 production release is freezed at [vendor's #4cb85b](`https://github.com/aptos-labs/aptos-core/commit/4cb85bc832b57acb26a627182163be6de2f9d83f`)

## Light Touch

Most of the Diem Platform Vendors publish code in "monorepo" code repositories. This means much of vendor code is tightly coupled. This is improving over time (there was a time when the Move Language was only developed in the Facebook/Diem/Diem repository).

We attempt to have a light touch approach with all our dependencies, to prevent the amount of forks we maintain.

## Extend and Layer

We first and Always try to work with unmodified dependencies.

- We always extend APIs where we import them. For example create new Diem Platform rust `traits` in Libra Framework to extend the functionality we need.
- Duplicate the code in the worst case, if it is a single function or small module.
- Mirror copies of critical policy code which doesn't have package management (Move standard libraries and diem framework)

## Vendorize Dependencies

Sometimes maintaining our own branch is inevitable, especially in newly emergent technologies.

When we must change an upstream dependency:

- visibility: we should aim to only ever make changes to the visibility of functions and objects in upstream code. This creates a clean abstraction between platform vendor tools, and libra-specific features.
- naming: sometimes, the vendors change names (:/) so we need to rename things back. sigh.
- decoupling: there are hard-coded terms in vendor code, such as urls, coin names, directories, commands, config files, etc. We need to separate those and make generic.

## Easy Rebases

- on our fork we consider our main branch `libra-framework`, which contains all of our changes. The changes should be linear so to be _rebased_ without conflict.
- the fork keeps the `main` branch which can be synced, and the changes are rebased onto `libra-framework` branch.
