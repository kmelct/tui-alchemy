# Alchemy TUI user guide

Use this page as the player-facing README for controls, reset, installation, and visual walkthroughs.

## In-game menu

Press `m` to open the minimal game menu.

| Menu item | Action |
| --- | --- |
| `resume` | Close the menu and return to the board. |
| `controls` | Open the controls submenu. Keep general controls there instead of crowding the main menu. |
| `reset game` | Open reset confirmation. Press `Enter` again to reset discoveries to the four starters, or `Esc` to go back. |

## Controls submenu

| Key or input | Action |
| --- | --- |
| `Arrow` keys, `h` `j` `k` `l` | Move through the atlas. |
| `Enter` | Select the highlighted element or menu item. |
| `1`-`9` | Select a visible atlas slot directly. |
| Drag or click | Move ingredients into recipe slots. |
| `m` | Open or close the menu. |
| `Esc` | Back out of a submenu, close the menu, or clear the current selection during play. |
| `q` | Quit. |

## Visual walkthrough

| Step | What to do | Screenshot |
| --- | --- | --- |
| Start | Open the game and review the atlas, progress rail, and recipe table. | <img alt="Alchemy TUI hero screen" src="screenshots/hero.png" width="320"> |
| Pick first ingredient | Select `Water` from the atlas. | <img alt="Water selected in the atlas" src="screenshots/01-select-element.png" width="320"> |
| Pick second ingredient | Select or drag `Fire` into the second recipe slot. | <img alt="Fire selected as the second ingredient" src="screenshots/02-select-second.png" width="320"> |
| Read the result | `Water + Fire` resolves into `Steam`, which joins the atlas. | <img alt="Steam discovered from Water and Fire" src="screenshots/03-get-result.png" width="320"> |
| Keep exploring | Use new elements as ingredients for deeper recipes. | <img alt="Populated board after several discoveries" src="screenshots/04-populated-board.png" width="320"> |

## Layout examples

| Terminal shape | Screenshot |
| --- | --- |
| Narrow | <img alt="Narrow terminal layout" src="screenshots/05-narrow.png" width="320"> |
| Large | <img alt="Large terminal layout" src="screenshots/06-xlarge.png" width="320"> |
| Short | <img alt="Short terminal layout" src="screenshots/07-height-24.png" width="320"> |
| Tall | <img alt="Tall terminal layout" src="screenshots/08-height-48.png" width="320"> |

## Other user documents

| File | Use |
| --- | --- |
| [`install.md`](install.md) | Install with the hosted script, Cargo, or binary archives. |
| [`release-v0.2.0.md`](release-v0.2.0.md) | Current release notes. |
| [`release-v0.1.0.md`](release-v0.1.0.md) | Previous release notes. |
