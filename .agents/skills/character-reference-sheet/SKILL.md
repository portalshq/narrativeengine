---
name: character-reference-sheet
description: Create a high-fidelity three-view character reference sheet from character samples. Use whenever a user asks to create a character, character sheet, or reference sheet.
---

# Character Reference Sheet

You are an expert character designer specializing in creating high-fidelity
character reference sheets. Analyze the provided character samples and generate
a single cohesive 3:2 image that serves as a professional reference.

## Layout Requirements

Divide the image into three distinct columns:

- **Detailed Portrait (left):** Close-up focus on the character's face. Capture
  the exact eye color, facial features, makeup, and head-worn accessories in
  high detail.
- **Full-Body Front View (center):** Head-to-toe view from the front. Clearly
  show the whole outfit, proportions, and frontal details.
- **Full-Body Back View (right):** Head-to-toe view from the back. Show hair
  styling, rear outfit details, and accessories not visible from the front.
- **Character Name (bottom right):** The character's correctly written name in
  large, plain text.

## Style and Fidelity

- **Exact style match:** This is not a hand-drawn or rough sketch sheet. Match
  the samples' material style, lighting, and rendering quality exactly, whether
  photorealistic, 3D-rendered, or a specific digital-art style.
- **Character consistency:** Preserve eye color, clothing textures, specific
  accessories, and color palette from the supplied materials.
- **Background:** Use a clean, flat, simple background that complements the
  design and keeps the character as the sole focus.

## Goal and Action

Produce an official-quality reference asset with complete stylistic and design
continuity from the supplied samples. Generate the image, then apply
`nap-update` to store it as the entity's `character_sheet` representation and
commit the updated manifest.
