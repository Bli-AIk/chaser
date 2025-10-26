---
name: Bug report
about: Report a bug or unexpected behavior in Chaser
title: 'bug: [Short description]'
labels: bug
assignees: ''

---

### 🐛 Problem Description
Provide a clear description of the issue. Include:
- Chaser version (if applicable)
- Type of issue: file monitoring / configuration / performance / unexpected behavior
- Configuration files or paths involved (optional, but helpful)

---

### 📝 Steps to Reproduce
Provide a minimal, self-contained example:

```bash
# Minimal reproducible example
chaser --config your-config.json --target your-target.json
```

1. What **command** or **configuration** did you use? What **parameters** did you provide?
2. Operating system (e.g., Windows 11, Ubuntu 22.04)
3. File system type and structure being monitored

---

### ✅ Expected Result
Describe what the correct behavior should be.
Example:
- Files should be monitored and configuration updated automatically.
- No error should be thrown during file operations.

---

### 📄 Actual Result / Logs
Include any errors, stack traces, or console output:

```text
// Paste your error message here
```

---

### ⚙ Additional Information
- Is the tool monitoring a large number of files?
- Are there permission issues with the files or directories?
- Does the issue appear on specific file types or paths?
- Any workarounds tried?
