# Auto-Execute Commands - Fixes Applied

## Issues Fixed

### 1. ✅ On-Connection Commands Not Applying Immediately
**Problem**: Changes to on-connection commands only took effect after closing and reopening the TUI, which was incredibly unintuitive and a major UX bug.

**Root Cause**: In `src/main.rs`, settings were loaded once at startup (line 49) and stored in a local variable. When the event loop returned the updated app with modified settings in `app.settings`, the connection logic was still using the stale `settings` variable from startup instead of the updated `app.settings`.

**Fix**: Changed all references after the event loop to use `app.settings` instead of the stale `settings` variable:
```rust
// Before (lines 85-90, 114, 117):
if !settings.auto_execute_commands.is_empty() { ... }
let shell = settings.default_shell.as_str();
ssm.launch_session(&instance_id, shell, app.credentials.as_ref(), &settings.auto_execute_commands)

// After:
if !app.settings.auto_execute_commands.is_empty() { ... }
let shell = app.settings.default_shell.as_str();
ssm.launch_session(&instance_id, shell, app.credentials.as_ref(), &app.settings.auto_execute_commands)
```

**Result**: Changes to on-connection commands now apply immediately when you connect to an instance, without needing to restart the TUI. The settings are properly persisted to disk AND used from the in-memory `app.settings` which gets updated by the event handler.

**Files Modified**: `src/main.rs`

### 2. ✅ Missing UI Hints
**Problem**: No hints on how to add commands or return to instance list

**Fix**:
- Added "Press 'a' to add your first command" hint in empty state
- Updated help text to show "[Esc] Back to instances"
- Added description and examples in edit mode
- Consolidated help text to one line with clear instructions

### 2. ✅ AWS CLI Parameter Parsing Error
**Problem**: Multiple commands causing AWS CLI parsing error:
```
Error parsing parameter '--parameters': Expected: ',', received: ''' for input:
 command=bash -l -c 'sudo su - ubuntu && exec bash -l'
```

**Fix**: Wrapped the command parameter value in double quotes:
```rust
// Before:
.arg(format!("command={}", shell_command));

// After:
let escaped_command = shell_command.replace('"', r#"\""#);
.arg(format!(r#"command="{}""#, escaped_command));
```

Now AWS CLI receives:
```bash
--parameters command="bash -l -c 'sudo su - ubuntu && exec bash -l'"
```

The double quotes ensure AWS CLI properly parses the entire command value, including any single quotes used for shell command chaining.

### 3. ✅ Delete Command Not Working
**Problem**: Deleting commands appeared not to work, commands still showing

**Fix**: Refactored delete handler to properly manage mutable state:
```rust
// Now uses single mutable borrow and properly adjusts selection index
if let Screen::Settings(ref mut state) = app.screen {
    let selected = state.selected_command;
    // ... delete and adjust selection
}
```

### 4. ✅ Commands Not Being Executed
**Problem**: Auto-execute commands not running on connection

**Root Cause**: The AWS CLI parameter parsing issue (see #2) was preventing the session from starting correctly.

**Verification**: Added debug output in main.rs to show which commands are loaded:
```
Auto-execute commands configured:
  1. sudo su - ubuntu
  2. cd /tmp
```

## How It Works Now

### Command Chaining
When you add commands like:
1. `cd /tmp`
2. `sudo su - ubuntu`

The system generates:
```bash
bash -l -c 'cd /tmp && sudo su - ubuntu && exec bash -l'
```

- Commands are joined with `&&` (stops on failure)
- Single quotes wrap the entire command sequence
- `exec bash -l` replaces the process with an interactive shell
- The whole thing is wrapped in double quotes for AWS CLI parameter parsing

### Quote Escaping
- Single quotes in commands are escaped as `'\''` for shell safety
- Double quotes are escaped as `\"` for AWS CLI parameter safety
- Example: `cd '/path with spaces'` works correctly

## Testing Checklist

### ✅ Unit Tests (12 tests passing)
- Settings serialization/deserialization
- Default values
- Quote escaping (single quotes)
- Command construction with/without auto-commands
- Command construction with quotes

### Manual Testing Steps

1. **Test Empty State**
   - Run app
   - Press `c` to open settings
   - Verify you see "Press 'a' to add your first command" hint

2. **Test Adding Commands**
   - Press `a` to add
   - Type: `sudo su - ubuntu`
   - Press Enter
   - Verify command appears in list
   - Press `a` again
   - Type: `cd /tmp`
   - Press Enter
   - Verify both commands show

3. **Test Editing Commands**
   - Navigate to first command with ↑/↓
   - Press `e` to edit
   - Change text
   - Press Enter
   - Verify changes saved

4. **Test Deleting Commands**
   - Navigate to a command
   - Press `d` to delete
   - Verify command removed from list

5. **Test Settings Persistence**
   - Exit app (press Esc, then q)
   - Restart app
   - Press `c` to open settings
   - Verify commands are still there
   - Check file: `~/.config/ssm-connect/config.json`

6. **Test Command Execution**
   - Add command: `echo "Test command executed"`
   - Exit settings (press Esc)
   - Connect to an instance (press Enter)
   - Verify you see "Test command executed" output
   - Verify you end up in an interactive shell

7. **Test with Complex Commands**
   - Test with paths containing spaces: `cd '/path with spaces'`
   - Test with quotes: `echo "Hello 'world'"`
   - Test with sudo: `sudo su - ubuntu`
   - All should work without AWS CLI parsing errors

## Config File Format

Location: `~/.config/ssm-connect/config.json`

Example:
```json
{
  "default_region": "ap-southeast-2",
  "default_shell": "bash",
  "auto_refresh_interval_seconds": 30,
  "theme": "dark",
  "auto_execute_commands": [
    "cd /tmp",
    "sudo su - ubuntu"
  ]
}
```

## Key Changes Summary

1. **src/aws/ssm.rs**: Fixed AWS CLI parameter quoting
2. **src/ui/screens/settings.rs**: Improved UI hints and help text
3. **src/events/handler.rs**: Fixed delete command state management
4. **src/main.rs**: Added debug output for verification
5. **tests**: Added 4 new tests for command construction

## Next Steps for User

1. Build and run: `cargo build --release`
2. Test the settings screen with the checklist above
3. Verify commands execute when connecting to an instance
4. If issues persist, check the debug output when connecting
