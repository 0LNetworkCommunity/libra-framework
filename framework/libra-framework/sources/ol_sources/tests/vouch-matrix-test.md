Let's first create a function which creates a simple vouch network. It may not be realistic, but it will allow to get some verification of the scoring calculations.
Let's create 110 users. 100 ordinary users, and 10 root of trust users. Then we'll create a vouch network that looks like a 10X10 matrix; number of vouches and depth of vouches.
Meaning: There will be 1 user that has 1 vouch from 1 root user (1st degree). 1 users that has 10 vouches from 10 root users (1st degree). Then at the other extremes, there will be 1 user with 1 vouch from one user that is 10 degrees removed from a root user. On the other extreme, 1 user that has 10 vouches from 10 users that 10 degrees from a root of trust.
We need to be sure to build out the vouch graph deliberately and sequentially. So:
1. Create root of trust users
2. Create first level of users
3. give vouches from root users to the first level users.
4. assert that the vouch state of the first level users are as we expect.
When we will loop through steps 2 to 4, to create subsequent levels.
------

Matrix Vouch Network Implementation
The test creates a 10x10 matrix-style vouch network with:

10 root of trust users
100 regular users arranged in 10 levels of depth
Each level has 10 users with varying numbers of vouches (1-10)
Network Structure
Root Level: 10 root users established in the root of trust
First Level (Level 0): 10 users receiving direct vouches from root users
User 1 gets 1 vouch from 1 root user
User 2 gets 2 vouches from 2 root users
...and so on until User 10 gets 10 vouches from all root users
Second Level (Level 1): 10 users receiving vouches from the first level
User 1 gets 1 vouch
User 2 gets 2 vouches
...and so on
This pattern continues for all 10 levels
Verification Logic
The test includes comprehensive verification logic to check that:

Horizontal Pattern: Within each level, users with more vouches have higher scores
Vertical Pattern: As we move further from the root users, scores decrease for users with the same number of vouches
Root Users: All root users are properly registered and have non-zero scores
The verification includes a 20% tolerance to account for rounding and network effects, ensuring the test doesn't fail due to minor variations while still validating the fundamental scoring patterns.

Debugging Features
I've included a score matrix printing function that outputs the entire score matrix for debugging. This will help visualize the patterns and identify any anomalies in the scoring.

Addressing Vouch Limits
The code respects the production vouch limits by:

Checking if a voucher has remaining vouches before attempting to create a vouch
Verifying that each step successfully creates the expected number of vouches
Using the test_helper_vouch_for function to bypass ancestry checks while respecting other limits
This approach will give you a well-controlled test environment to verify the page rank algorithm's behavior while working within the system's constraints.
