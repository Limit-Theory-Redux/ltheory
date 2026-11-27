# UI icons

The game map icons (stations, ships by class, planets, asteroids, gates and
more) are drawn by BFett in Inkscape. `game-map-icons.svg` is that drawing;
the "Game Map Icons" page holds the icons, each with its name beside it.

The game doesn't read SVG. `make_map_icons.py` renders the icons into white
textures with the shape in the alpha channel, which the HUD tints per colour
(`Gui:setImageColor`):

- `res/tex2d/map-icon/<name>.png`: 128 x 128, the icon's longer side 120
  pixels, centred;
- `res/tex2d/map-icon/<name>-glow.png`: 64 x 64, the icon's longer side 32
  pixels, blurred: a soft glow to draw under the icon.

Load one with `Tex2D.Load("map-icon/<name>")`. Drawn much smaller than
128 pixels it needs mip levels to stay smooth: see the HmGui README's Images
section (`engine/lib/phx/src/ui/hmgui/README.md`), and `iconTexture` in
`script/UI/HmGui/Components/Hud/SensorHud.lua`.

## Regenerating

From the repository root (needs Python 3 with numpy and Pillow):

```sh
python -I tools/ui-icons/make_map_icons.py
```

Then commit the changed textures with the SVG.

## Adding or changing an icon

Each icon in the `ICONS` table of `make_map_icons.py` is a name and the ids of
the SVG shapes it is made of; a group's id takes every shape in it. Inkscape
shows an object's id in the XML editor (Edit > XML Editor) and in Object
Properties. Edit the drawing, keep or update the ids in `ICONS`, and
regenerate.

The script reads paths, rectangles, circles, ellipses, polygons, lines and
`use` references, with their transforms and inherited fill and stroke; fills
follow their fill rule, and a radial gradient fills a circle from its centre.
Text, filters, masks and clip paths are left out. `svgshapes.py` is the SVG
reader.
