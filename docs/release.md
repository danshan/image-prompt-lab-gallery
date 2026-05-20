# Release

This project uses a macOS-only Tauri release workflow for the first desktop distribution path.

## Signing Boundary

The current workflow uses macOS ad-hoc signing. It does not require an Apple Developer certificate and does not run notarization.

Ad-hoc signing is not Developer ID signing. It does not guarantee that a downloaded app can be opened for the first time without macOS Gatekeeper intervention. If the product needs that first-open experience, add Developer ID signing and notarization in a separate release hardening change.

Tauri updater signing is separate. It verifies that update artifacts were produced by the holder of the updater private key. It does not replace Apple notarization.

## Generate Updater Keys

Generate the updater key pair on a trusted local machine:

```text
cd apps/desktop
npm run tauri signer generate -- -w ~/.tauri/image-prompt-lab.key
```

Enter a strong password when prompted. The generated files have different purposes:

- `~/.tauri/image-prompt-lab.key`: private key. Keep it secret. Do not commit it.
- `~/.tauri/image-prompt-lab.key.pub`: public key. Copy its content into `apps/desktop/src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
- The password: store it in a password manager and GitHub secrets.

The repository contains the updater public key used by release builds. Replace it only when rotating the updater signing key pair.

## GitHub Secrets

Configure these repository secrets:

```text
TAURI_SIGNING_PRIVATE_KEY
TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

`TAURI_SIGNING_PRIVATE_KEY` is the private key file content, not the path. You can inspect the file locally and paste its full content into the secret.

No Apple secrets are required for the current workflow:

```text
APPLE_ID
APPLE_PASSWORD
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
```

## Version Discipline

Keep these versions aligned before pushing a release tag:

- `apps/desktop/package.json`
- `apps/desktop/src-tauri/tauri.conf.json`

The tag should match the desktop app version:

```text
vX.Y.Z
```

## Publish

Run local checks first:

```text
cd apps/desktop
npm run build
cd ../..
cargo check -p imglab-desktop
```

Then commit the version bump and push a tag:

```text
git tag vX.Y.Z
git push origin vX.Y.Z
```

The release workflow also supports manual dispatch from GitHub Actions. Draft releases are useful for asset inspection, but installed apps using the latest-release updater endpoint only consume non-draft releases.

## Verify Release Assets

After the workflow completes, inspect the GitHub Release assets. A valid updater release should include:

- macOS app bundle assets.
- Tauri updater artifact, currently `Image Prompt Lab.app.tar.gz`.
- Signature file.
- `latest.json`.

## Verify Auto Update

1. Install an older release.
2. Publish a newer non-draft release.
3. Open the older installed app.
4. Confirm startup does not block the main workflow.
5. Open `Settings / Updates`.
6. Click `Check for Updates`.
7. Confirm the new version is shown.
8. Click `Download and Install`.
9. Click `Restart`.
10. Confirm the restarted app reports the new version.
