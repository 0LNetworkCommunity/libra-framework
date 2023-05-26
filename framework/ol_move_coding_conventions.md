# 0L Move Coding Conventions
For core system code, we observe the following patterns.

## Belt and Suspenders

If you see "belt and suspenders" in the code, we are flagging the cases below. It means "yes this appears redundant, but we don't trust your fingers".

 Move is a new languge, and devs won't have developed intuitions about it. Here we are safeguaring against developer errors, not malicious hacks. It's not only for logic bugs, typos, and incomplete work. Some  Move lang features make it that a fat finger, a git merge, a documentation commit, could expose a public api.

For these cases you will see "belt and suspenders" implementations. They will seem ineficient and paranoid. But over time and over many developers, one of these may save from a catastrophic error.

Wherever there are "critical mutations" i.e. state changes related to consensus, accounts values, or privilege escalation, we place multiple redundant checks on functions.


1. Always authorize public functions: This should be obvious. Public functions which call critical mutations are authorized by `signer`s that are either the account holder or the `root` account.

2. Friends: Assume a public function with critical mutation must be restriced by a `friend` permission, such that third party modules and scripts can only call through a limit number of paths.

3. Write the private `fun` first: State mutations for account values, authorization, and consensus critical are done in private functions. 

4. Authorize private fun: The private functions which cause mutations also need to be authorized. There are exceptions, but only ones they are deeply nested in other private functions that are authorized. If you accidentally make a private function public, there won't be an issue.

5. Authorize test helpers: The `#[test_only]` annotations are especially dangerous. If a developer accidentally removes an annotation (or places a line break in the wrong place), then the helper function may be exposed as a public function. Treat test functions as entry points that need to be authorized by `root`. We have seen these errors by the best Move develoeprs, e.g. in Move standard library and vendor code.

6. Write formal verification `specs`: They don't need to be complicated. Even simple ones will save from a catastrophic deployment. An important patters is to use them within module code, for example in `if` flows and `while` loops to check that a condition isn't being met.

7. Use `assert!` liberally:  Functions that only have USER transactions should always abort and return errors for known exceptions. An `assert!` shouldn't prevent you from writing a `spec`, assume you need to do both. Note this does not apply to `root` functions: `assert!` could cause an abort during consensus critical code and that will be fatal. Alway use flow control and return early. (There is an exception for genesis of the network in `genesis.move`). 

## Denial of Service Precautions

# Avoid looping on public calls
There is an easy vector for DOS related to list looping. When users can increment a list at low cost via a public function you've opened a vector for attack. In those cases there is likely another public function which iterates over that list. If there are cheap calls to both functions, then there is an easy DoS attack. With Move there is an even more dangerous vector:  `#[view]` functions are free to call via the REST api. So if it loops over a list, you have basically created a free DoS vector.

Where this is unavoidable, a rate-limit needs to be placed on caller which increments the list.

V7 TODO: rate limits.
