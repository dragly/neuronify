# Task 8: Scenario Briefing UI

When a scenario loads, display the briefing to the player before gameplay begins.

## Content

Use the parsed scenario metadata:
- Scenario name (large title)
- Briefing text (the `data-briefing` value — explains the situation and hints at the solution)
- Objective text (the `data-objective` value — what the player must do)

## Interaction

- Display as an overlay on top of the rendered map (so the player can see the terrain behind the briefing)
- Player dismisses with a tap/click or a "Begin" button
- Game simulation is paused until the briefing is dismissed

## Visual style

Match the game's existing UI aesthetic. The briefing should feel like a military intelligence dossier — clean monospace text on a semi-transparent dark panel.
