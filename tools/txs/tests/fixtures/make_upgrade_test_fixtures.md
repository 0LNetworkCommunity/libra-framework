
## Generate these fixtures

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

#### 1. Copy the `all_your_base.move` file to `./framework/move-stdlib`
Remember to remove it later.

#### 2. execute the governance script generator
You can output the artifacts directly to this fixture dir
```
cd ./framework

cargo r --release upgrade  --output-dir ./test_upgrade" \
        --framework-local-dir <project_root_here>/framework/
```

#### 3. Check files
If everything went well you'll have a `script.mv` and a `script_sha3` file here. The script_sha3 should be a different hash that you've had before if there are any changes to the MoveStdlib.

#### 4. Run the test
The test will call a view function which should return hex enconded "us", which is "0x7573".