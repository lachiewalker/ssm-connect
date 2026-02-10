# Verification Guide for Auto-Execute Commands

## Quick Test Procedure

### 1. Build and Run
```bash
cargo build --release
./target/release/ssm-connect
```

### 2. Navigate to Settings
- From the instance list, press `c`
- You should see the settings screen with clear instructions

### 3. Add a Test Command
- Press `a` to add a command
- Type: `echo "Auto-command executed successfully!"`
- Press `Enter` to save
- You should see it listed as "1. echo "Auto-command executed successfully!""

### 4. Add a Second Command (to test the fix for multiple commands)
- Press `a` again
- Type: `pwd`
- Press `Enter`
- Both commands should now be visible

### 5. Test Deletion
- Navigate to command 2 with `↓` arrow
- Press `d` to delete
- Command 2 should disappear immediately
- Only command 1 should remain

### 6. Exit Settings
- Press `Esc` to return to instance list
- The help text should have shown "[Esc] Back to instances"

### 7. Verify Persistence
- Quit the app (`q` or `Esc`)
- Check the config file:
  ```bash
  cat ~/.config/ssm-connect/config.json
  ```
- You should see your command in the `auto_execute_commands` array

### 8. Test Command Execution
- Restart the app
- Connect to a running instance (press `Enter` on an instance)
- You should see:
  ```
  Connecting to instance: i-xxx
  Auto-execute commands configured:
    1. echo "Auto-command executed successfully!"
  ```
- After connection, you should see the output: `Auto-command executed successfully!`
- You should then be in an interactive shell

### 9. Test the Complex Command That Failed Before
- Open settings (`c`)
- Delete the test command if still there (`d`)
- Add: `sudo su - ubuntu`
- Save and exit
- Connect to an instance
- **Previously this failed with "Error parsing parameter"**
- **Now it should work!** You should successfully switch to the ubuntu user

## What Changed to Fix the Issues

### Issue #1: UI Hints
**Before**: No indication of how to use the settings screen
**After**:
- Empty state shows "Press 'a' to add your first command"
- Help text shows all available keys including "Back to instances"
- Edit mode shows description and examples

### Issue #5: AWS CLI Parsing Error (Most Critical)
**Before**:
```bash
--parameters command=bash -l -c 'sudo su - ubuntu && exec bash -l'
```
AWS CLI parser failed because it saw the `'` as a new token.

**After**:
```bash
--parameters command="bash -l -c 'sudo su - ubuntu && exec bash -l'"
```
Double quotes wrap the entire value, AWS CLI parser correctly handles it.

### Issue #4: Delete Not Working
**Before**: State management issue caused stale state
**After**: Proper mutable borrow and index adjustment

### Issue #3: Commands Not Executing
**Cause**: Issue #5 prevented the session from starting
**Fix**: Now that AWS CLI parsing works, commands execute correctly

## Troubleshooting

### Commands not appearing in list after saving
- Check `~/.config/ssm-connect/config.json` - is the command there?
- If yes but not showing: file a bug (UI refresh issue)
- If no: file a bug (save issue)

### Commands not executing on connection
- Check for the debug output: "Auto-execute commands configured:"
- If not shown: settings not loading (check config file)
- If shown but not executing: check AWS CLI version and SSM plugin

### AWS CLI errors
- If you still get "Error parsing parameter": check the exact command being sent
- Run with RUST_LOG=debug to see full command
- Verify AWS CLI version: `aws --version` (should be 2.x)

### Command fails but shell doesn't start
- This is expected behavior - we use `&&` to chain commands
- If a command fails, subsequent commands don't run
- But the interactive shell should still start (due to the fix)

## Expected Behavior

1. **Single command**: Executes, then starts interactive shell
2. **Multiple commands**: Execute in order (chained with `&&`), then interactive shell
3. **Command with failure**: Stops at failure, but shell still starts
4. **Empty command list**: Direct to interactive shell (no auto-commands)
5. **Complex quotes**: Properly escaped and handled

## Success Criteria

- ✅ Can add/edit/delete commands in UI
- ✅ Commands persist across restarts
- ✅ Commands execute on SSM connection
- ✅ No AWS CLI parsing errors
- ✅ Interactive shell works after commands
- ✅ UI shows clear instructions
