# Gallery Lightbox And Thumbnail Frame Design

## Goal

Fix two Gallery image presentation issues:

- Clicking the image in the right-side Inspector should open the complete original image inside the app.
- Gallery thumbnails and Inspector thumbnails should not show the inner square frame overlay.

## Current Context

The desktop UI is implemented in `apps/desktop/src/main.tsx` and `apps/desktop/src/styles.css`.

Gallery cards and the Inspector hero both render images through the shared `Thumbnail` component. The image source is resolved with `convertImagePath()`, which delegates to Tauri `convertFileSrc()` in the desktop runtime. The visible inner square is produced by the shared `.thumbnail::after` pseudo-element in CSS.

The Gallery card image is already inside a button whose click selects the asset. The Inspector image has no separate preview action today.

## Chosen Approach

Use a focused in-app lightbox opened from the Inspector thumbnail.

The lightbox should:

- Open only when the selected asset has an `imagePath`.
- Render above the workbench as a fixed overlay.
- Display the original image source with `object-fit: contain`, preserving the image aspect ratio and showing the whole image within the viewport.
- Close from a close button, backdrop click, or `Escape`.
- Keep Gallery card clicks unchanged, so clicking a card still selects the asset.

Remove the shared `.thumbnail::after` pseudo-element so neither Gallery nor Inspector thumbnails render the inner square frame.

## Rejected Alternatives

### Metadata lightbox

A lightbox with an image and side metadata panel would provide more context, but it adds layout and state that are not required for this fix. The Inspector already shows metadata next to the image, so duplicating it in the overlay is unnecessary.

### System image viewer

Opening the file in the system default image viewer would be simple, but the requirement is an in-app overlay. It also introduces platform-specific behavior and moves the user out of the workbench.

### Open from Gallery card click

Using Gallery card image clicks for preview would conflict with the existing selection behavior. Keeping preview scoped to the Inspector preserves the current Gallery interaction model.

## Implementation Shape

Add lightbox state near the existing selected asset and detail state in `App`.

Pass a preview callback into `Inspector`, and make the Inspector hero thumbnail clickable only when the selected asset has an `imagePath`. This can be done by wrapping `Thumbnail` in a button with a dedicated class, while keeping `Thumbnail` itself presentation-only.

Render an `ImageLightbox` component near the workbench root when preview state is present. The component receives the image path and accessible label, converts the image path through the existing `convertImagePath()` helper, and owns overlay close behavior.

CSS should define:

- A fixed overlay with a dark translucent backdrop.
- A bounded image area using viewport-relative max dimensions.
- `img` styles that preserve aspect ratio and show the full image.
- A small close button with stable dimensions.
- A button reset style for the Inspector thumbnail trigger.

Remove `.thumbnail::after`.

## Error Handling And Edge Cases

- If `imagePath` is missing, the Inspector thumbnail remains non-clickable.
- If the image fails to load, the browser image error state is acceptable for this narrow fix. A richer error placeholder can be added later if image integrity handling is expanded.
- `Escape` handling must be registered only while the lightbox is open and cleaned up on close or unmount.
- Backdrop click should close the overlay, but clicks on the image or close button should not accidentally bubble as backdrop clicks.

## Testing

Manual verification:

- Open Gallery and select an asset with an image.
- Confirm Gallery card click still selects the asset.
- Confirm the Inspector thumbnail opens the lightbox.
- Confirm the full image is visible without cropping.
- Confirm close button, backdrop click, and `Escape` close the lightbox.
- Confirm no inner square frame appears on Gallery thumbnails or the Inspector thumbnail.

Automated coverage:

- Existing state tests do not cover DOM behavior. This change can be verified manually unless a React component test harness is introduced.
- Run the desktop frontend checks already used by the project after implementation.

## Scope

In scope:

- Inspector-triggered in-app full image preview.
- Removal of thumbnail inner frame overlay.
- CSS and React changes only.

Out of scope:

- Metadata side panel inside the lightbox.
- Zoom, pan, rotate, download, or open-in-file-manager controls.
- Changing Gallery card click semantics.
- Backend, database, or resource library changes.
