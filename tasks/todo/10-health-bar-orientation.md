# Task 10 — Health Bar Camera Alignment

## Problem

Health bars are rendered as quads/lines in world space but are not oriented relative to the camera. They must always face the player by being aligned with the camera's right vector, so they appear perpendicular to the view direction regardless of camera angle.

## Acceptance Criteria

- Health bar quads are rotated so their long axis is parallel to the camera right vector.
- They remain upright (no tilt) as the camera pans or zooms.
- No visual change to health bar colour, size, or position — only orientation is fixed.
