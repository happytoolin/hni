## SpecStory

## Untitled (2/14/2025, 3:04:49 PM)

_**User**_

remove the prealpha thing

---

_**Assistant**_

I'll help you remove the pre-release related inputs and functionality from the GitHub Actions workflow file. I'll edit the release.yml file to remove the pre-release options and related logic.

---

_****_

```diff
            - patch
          default: patch
-       is_prerelease:
-         description: "Is this a pre-release?"
-         required: false
-         type: boolean
-         default: false
-       prerelease_id:
-         description: "Prerelease identifier (e.g., alpha, beta, rc). Used only if is_prerelease=true."
-         required: false
-         type: string
-         default: "alpha"
  
  permissions:
                ;;
            esac
- 
-           # Add prerelease suffix if needed
-           if [ "${{ github.event.inputs.is_prerelease }}" = "true" ]; then
-             NEW_VERSION="${NEW_VERSION}-${{ github.event.inputs.prerelease_id }}"
-           fi
  
            # Update Cargo.toml
              --notes-file release_notes.md \
              --draft=false \
-             --prerelease=${{ github.event.inputs.is_prerelease }} \
              artifacts/*/*
```

---

_****_

I've removed all pre-release related functionality from the GitHub Actions workflow file.

---

---
