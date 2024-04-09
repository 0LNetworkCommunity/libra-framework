# Drop User Lib
A decent amount of time was spent figuring out how to drop accounts from
a Diem chain without requiring a halt. It will not be used currently.
These files are modifications of the Move framework source. They are kept for future reference.

Some system Structs (as in account.move) needed to have a 'drop' capability, and
that's not desirable for ongoing chain. So one might use it during a specific
offline operation, or intervention.
