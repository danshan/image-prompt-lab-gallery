# Gallery Masonry Interaction Design

## Summary

This change adjusts the desktop Gallery asset board layout and click behavior without changing Rust core read models, resource library data, or asset detail semantics.

The Gallery should become a fixed-width masonry board that preserves each image's natural aspect ratio where possible. Very tall images should be visually capped so a card image never exceeds a `2:3` width-to-height ratio. When an image is taller than that cap, the preview should crop from the bottom and preserve the top of the original image.

Card interaction should split preview and detail intent:

```text
Image click -> open original image lightbox
Card chrome / blank area click -> select asset and show Inspector detail
```

## Goals

- Make the Gallery easier to scan across mixed image ratios.
- Preserve original image aspect ratios for normal landscape, square, and portrait images.
- Prevent very tall images from dominating the masonry flow by capping visual preview height at `2:3`.
- Preserve the top of over-tall images when the preview is capped.
- Separate original image preview from Inspector detail selection.
- Keep the implementation narrow and local to the desktop Gallery UI.

## Non-Goals

- Do not change Gallery query semantics.
- Do not change Rust core read models or database schema.
- Do not introduce Gallery virtualization.
- Do not add drag ordering, keyboard spatial navigation, or a new selection model.
- Do not replace the existing image lightbox behavior outside the Gallery card click target.

## Selected Approach

Use CSS column-based masonry with fixed card width and card-level click delegation.

Gallery cards should render in a masonry flow using fixed-width columns. Each card remains an image-first item with metadata and actions below the image. The image preview area should compute its display ratio from asset dimension metadata when available:

- If `width` and `height` are available and the ratio is no taller than `2:3`, use the natural ratio.
- If the image is taller than `2:3`, display it in a `2:3` preview frame.
- If dimensions are missing, use a stable fallback ratio, such as the current `4:3` preview.

For capped over-tall images, use top-aligned cropping so the top of the source remains visible. In CSS terms, this means the preview image should use an equivalent of:

```text
object-fit: cover
object-position: top center
```

The card should remain the selected visual unit, but click targets should be explicit:

- The image preview target opens the original image in the existing lightbox.
- The non-image card body selects the asset and loads Inspector detail.
- Nested controls such as Review and batch selection stop propagation and keep their current action semantics.

## Alternatives Considered

### CSS Grid With Measured Row Spans

This would measure each loaded image and assign `grid-row-end` spans. It gives stronger control over DOM order and future spatial navigation, but adds resize, image load, and placeholder complexity. That complexity is not needed for this narrow interaction change.

### Masonry Layout Dependency

A dedicated masonry library could handle more edge cases, but it adds dependency and integration surface for a layout that can be handled with CSS in the current scope.

## Component Boundaries

The implementation should stay within the desktop frontend:

- `GalleryWorkspace`: owns card click target wiring and selection callback usage.
- `Thumbnail`: owns preview ratio calculation and lightbox trigger wiring.
- `styles.css`: owns masonry layout, fixed card width, capped image frame, and top-aligned cropping.
- Existing lightbox state and `ImageLightbox` should be reused rather than duplicated.

No backend command or core model should be added for this change.

## Interaction Rules

1. Clicking a Gallery image opens the original image lightbox.
2. Clicking Gallery card chrome, title, metadata, tags, footer, or blank space selects the asset and updates Inspector detail.
3. Clicking Review starts review behavior and does not also select or preview the asset.
4. Clicking the card checkbox toggles batch selection and does not also select or preview the asset.
5. The selected card styling still applies to the full card.
6. Keyboard focus should remain visible on actionable controls.

## Layout Rules

1. Gallery uses fixed card width masonry rather than equal-height grid tracks.
2. Normal images preserve natural aspect ratio from known dimensions.
3. Over-tall images are capped at `2:3`.
4. Over-tall image previews preserve the top of the source and crop from the bottom.
5. Missing dimensions use a stable fallback ratio.
6. The layout must remain usable at the existing compact desktop target.

## Error And Fallback Behavior

If an asset has no image path, the card should render the existing placeholder styling and still allow card-body selection. If dimension metadata is missing or invalid, the preview should use the fallback ratio rather than fabricating dimensions.

If lightbox opening is requested for an asset without an image path, the click should be inert or disabled at the image target level.

## Verification

Run the frontend build:

```text
npm run build
```

Perform a browser smoke test against the desktop UI:

```text
Gallery with landscape, square, portrait, and over-tall assets
Image click opens original lightbox
Card non-image click selects the asset and loads Inspector detail
Over-tall preview is capped at 2:3 and preserves the top
Review button does not trigger selection or lightbox
Checkbox toggles batch selection only
Compact desktop width keeps the masonry board usable
```

## Risks

CSS column masonry can make visual reading order differ from strict row-major grid order. This is acceptable for this change because the Gallery is currently a scan-first image board and this iteration does not add spatial keyboard navigation or drag ordering.
