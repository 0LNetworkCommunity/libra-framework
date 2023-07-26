
# DEPENDENCY MANAGEMENT
As stated elsewhere, 0L's innovations are in the middleware and application layers primarily in regards to economic policy. We rely heavily on (wonderful and cutting edge) infrastructure dependencies.

Our code management strategy means we separate some concerns and have clear APIs between each major system component.
- `Libra Framework`: is responsible for the framework source and tools to test and deploy etc. This is mostly original source that wraps or relies heavily on Move Language and Diem Platform (see below).
- `Carpe` is a reference light miner and wallet. This is original source imports our libra-tools and tower.
- `Tower`: is the verifiable delay function proof-of-work system 0L used to bootstrap and mint all the coins up to V7. This is original source, lightly uses `libra-tools`.
- `Libra-Tools`: low level CLIs and libraries for wallets, transactions, queries. This is original source that relies heavily on Diem Platform.
- `Diem Platform`: is infrastructural network node source (database, consensus, networking, configs, testing). There are linear commits to Facebook's Diem and today is maintained primarily by Matonee Inc (Aptos).
- `Move Language`: is the language spec, standard library, virtual machine, and dev tools. There are linear commits to Facebook's Move source, and today is primarily maintained by Mysten Labs Inc (Sui).

# Commit History

## Hello World
The commit history of `ol/diem-platform` will show that 0L forked Facebook's Libra (later renamed Diem) on their launch day in June 2019.

## 2019
A majority of the work in 2019 existed in separated now deprecated projects; `ol/movemint` for example, based on Cosmos SDK tooling.

On the `diem-platform` repo the majority of the work begain in Q4 2019, where we maintained the Diem monorepo structure.

We added our cryptography ceremony (Towers) and policy layers to mostly *.mvir (Yes, we were wrting in the Move Intermediary Representation before the Move language existed properly!).

## 2020
The V1 0L public mainnet was launched in Q3 2020 (TODO: Links to commits). Fun fact, you can see our crypto seeds based on contemporaneous news of the pandemic period.

## 2021
That code was based on commit (TODO: XYZ) from Facebook Diem v 1.4. The V2 Public Network happened in November 2021, based on Facebook Diem 1.5 (TODO: Commit).

## 2022
The Move Language repo became decoupled into its own repo in january 2022 (TODO: commit history).

Matonee's Aptos Platform forked Diem at commit XYZ in March 2022. (Note: the commit history appears to have rebased and commit hashes have changed).

0L V6 was based on the end-on-of-the-line Facebook commit XYZ (Same as Aptos).

Mysten Lab's Sui platform forked Move Language at commit XYZ in (TODO October 2022?)

## 2023
0L V7 pulled in the (excellent and ninja) maintenance upgrades from Matonee since commit XYZ.

0L separated our `libra` repo (with linear history to Facebook's first commit) into `libra-framewok` and `diem-platform`.

## Light Touch

Most of the Diem Platform Vendors publish code in "monorepo" code repositories. This means much of vendor code is tightly coupled. This is improving over time (there was a time when the Move Language was only developed in the Facebook/Diem/Diem repository).

We attempt to have a light touch approach with all our dependencies, to prevent the amount of forks we maintain.

## Extend and Layer
We first and Always try to work with unmodified dependencies.

- We always extend APIs where we import them. For example create new Diem Platform rust `traits` in Libra Framework to extend the functionality we need.
- Duplicate the code in the wost case, if it is a single function or small module.
- Mirror copies of critical policy code which doesn't have package management (Move standard libraries and diem framework)


## Vendorize Dependencies
Sometimes maintaining our own branch is inevitable, especially in newly emergent technologies.

When we must change an upstream dependency:
- visibility: we shoul aim to only ever make changes to the visibility of functions and objects in upstream code. This creates a clean abstraction between platform vendor tools, and libra-specific features.
- naming: sometimes the vendors change names (:/) so we need to rename things back. sigh.
- decoupling: there are hard-coded terms in vendor code, such as urls, coin names, directories, commands, config files, etc. We need to separate those and make generic.

## Easy Rebases
- on our fork we consider our master branch `libra-framework`, which contains all of our changes. The changes should be linear so to be *rebased* without conflict.
- the fork keeps the `main` branch which can be synced, and the changes are rebased onto `libra-framework` branch.