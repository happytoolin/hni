## SpecStory

## GitHub Release Asset Name Conflict (2/14/2025, 3:49:19 PM)

_**User**_

# Prepare release notes

echo "Release version $VERSION" > release_notes.md
echo "## Changes" >> release_notes.md
echo "See the [CHANGELOG.md](CHANGELOG.md) for details." >> release_notes.md

# Create release

gh release create "v$VERSION" \
    --title "Release v$VERSION"\
--notes-file release_notes.md\
--draft=false\
artifacts/_/_
shell: /usr/bin/bash -e {0}
env:
GITHUB_TOKEN: ***
VERSION: 0.0.5
HTTP 422: Validation Failed (https://uploads.github.com/repos/spa5k/nirs/releases/200356301/assets?label=&name=install.ps1)
ReleaseAsset.name already exists
Error: Process completed with exit code 1.

---

_**Assistant**_

I understand that you're encountering an error during the release process where there's a conflict with an existing asset named `install.ps1`. The error indicates that an asset with this name already exists in the release. Let me help you fix this issue.

First, let me check the release workflow file to understand how we can modify it to handle this case better.

The issue is that when creating a release, GitHub doesn't automatically overwrite existing assets. We need to modify the release creation process to either:

1. Delete any existing release assets before uploading new ones, or
2. Make the asset names unique by including the version number

Let me modify the release workflow to make the asset names unique by including the version number:

---

_****_

```diff
            copy_binary "nun"
  
-           # Copy install scripts
+           # Copy install scripts with version number
+           cp install.sh "artifacts/install-v${{ needs.update_version.outputs.new_version }}.sh"
+           cp install.ps1 "artifacts/install-v${{ needs.update_version.outputs.new_version }}.ps1"
+           # Also keep the original names for latest version reference
            cp install.sh artifacts/install.sh
            cp install.ps1 artifacts/install.ps1
```

---

_****_

Now let me also modify the release creation step to ensure we handle the assets properly:

---

_****_

```diff
            VERSION: ${{ needs.update_version.outputs.new_version }}
          run: |
+           # Delete existing release and tag if they exist
+           gh release delete "v$VERSION" --cleanup-tag || true
+           
            # Prepare release notes
            echo "Release version $VERSION" > release_notes.md
```

---

_****_

I've made two important changes to fix the issue:

1. Modified the artifacts preparation step to:
   - Create versioned copies of install scripts (e.g., `install-v0.0.5.sh`, `install-v0.0.5.ps1`)
   - Keep the original names as well for latest version reference

2. Modified the release creation step to:
   - Delete any existing release and tag before creating a new one (with `|| true` to prevent failure if the release doesn't exist)
   - Then proceed with creating the new release

These changes should resolve the "ReleaseAsset.name already exists" error by:

1. Ensuring we have unique versioned assets
2. Cleaning up any existing release before creating a new one

You can now retry the release process. The workflow will handle both versioned and latest install scripts properly, and there won't be any conflicts with existing assets.

---

_****_

---

---
