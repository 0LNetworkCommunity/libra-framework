
## Generate these fixtures
Making these fixtures is prone to error, so we have some helpers.

TODO: fixtures should be done pre-test in a build.rs

## TL;DR
```
RUST_MIN_STACK=104857600 cargo t -- make_the_upgrade_fixtures --include-ignored
```

## Fixtures

We want to test that we can modify code in the 0x1 address and upgrade successfully.

To do this we will create a framework upgrade script for MoveStdlib. We will add a module and function that would not otherwise not be there; "all your base"

```

module std::all_your_base {
  #[view]
  public fun are_belong_to(): vector<u8> {
    b"us"
  }
}
```

#### 1.Run the test: make_the_upgrade_fixtures()
This will copy the `all_your_base.move` temporarily to file to `./framework/move-stdlib`. It will then compile the artifacts and later remove the file.


The interface is just using cargo test. We need to also increase the stack size, because often it overflows. We have a makefile hack for this.

```
	RUST_MIN_STACK=104857600 cargo t -- make_the_upgrade_fixtures --include-ignored
```

Theres a makefile for this

```
make upgrade_fixtures
```

#### 2. Check files
If everything went well you'll have a `script.mv` and a `script_sha3` file here. The script_sha3 should be a different hash that you've had before if there are any changes to the MoveStdlib.

#### 3. Run the test
Any test that call the newly deployed view function which should return hex enconded "us", which is "0x7573".