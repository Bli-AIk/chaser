### PR Type

*Please check the type of this PR (you can select multiple)*

- [ ] 🐞 Bug fix
- [ ] ✨ New feature
- [ ] 📚 Documentation change
- [ ] 🎨 Code style change (such as formatting, renaming)
- [x] ♻️ Refactoring
- [ ] ⚡️ Performance improvement
- [x] 🧪 Adding or updating tests
- [ ] 🤖 CI/CD (Changes to CI/CD configuration)
- [ ] 📦 Other changes

### Related Issue

N/A - This is a refactoring to simplify the CLI interface based on user feedback and project requirements.

### What does this PR do?

This PR removes the `sync` and `update-path` commands from the CLI interface to simplify the user experience and focus on core functionality:

- **Removed `sync` command**: Previously allowed users to manually start path synchronization monitoring
- **Removed `update-path` command**: Previously allowed users to manually update paths in target files
- **Updated CLI definitions**: Removed command definitions from both production and test CLI builders
- **Cleaned up internationalization**: Removed unused translation keys from both English and Chinese locale files
- **Updated tests**: Removed related unit tests and integration tests for the deleted commands
- **Preserved core functionality**: 
  - The `status` command still works to show synchronization status
  - Automatic path synchronization during file monitoring continues to work
  - The `PathSyncManager` module is preserved for internal use

### How to test?

1. **Build and test the application**:
   ```bash
   cargo build --release
   cargo test
   ```

2. **Verify commands are removed**:
   ```bash
   # Check help output - sync and update-path should not appear
   ./target/release/chaser --help
   
   # Try to use removed commands - should show error
   ./target/release/chaser sync
   ./target/release/chaser update-path old new
   ```

3. **Verify existing functionality still works**:
   ```bash
   # Status command should still work
   ./target/release/chaser status
   
   # Basic commands should still work
   ./target/release/chaser list
   ./target/release/chaser add ./test-path
   ```

4. **Run all tests to ensure no regression**:
   ```bash
   cargo test --all-targets
   ```

### Screenshots or Videos

N/A - This is a CLI refactoring without UI changes.

### Additional Information

**Breaking Changes**: This is a breaking change that removes two CLI commands. Users who were using `sync` or `update-path` commands will need to rely on:
- The automatic path synchronization that happens during file monitoring
- The `status` command to check synchronization status

**Files Modified**:
- `src/cli.rs` - Removed command definitions and parsing logic
- `src/main.rs` - Removed command handlers and implementation functions
- `locales/en.yaml` - Removed unused translation keys
- `locales/zh-cn.yaml` - Removed unused translation keys  
- `tests/integration_tests.rs` - Removed tests for deleted commands

**Rationale**: The sync and update-path commands were deemed unnecessary as:
1. Path synchronization happens automatically during file monitoring
2. Manual path updates can be done by editing target files directly
3. Simplifying the CLI reduces complexity and maintenance burden

**Testing**: All existing tests pass (71 unit tests + integration tests + performance tests), confirming that no core functionality was broken.