# Generates the walking-Claw'd pixel-art sprite for the plasmoid:
#   plasmoid/contents/icons/claude-walk.png   (4-frame horizontal sheet, 13x12/frame)
#   plasmoid/contents/icons/claude-still.png  (standing frame, for the popup)
#
# Modelled on Anthropic's "Claw'd" mascot: a flat terracotta block with two arms
# out the sides, two solid black square eyes, and two legs. Animation = legs walk
# (alternating) + eyes dart right.
#
# To use the OFFICIAL asset instead: drop a 4-frame sheet at claude-walk.png and
# match frameCount/frameWidth/frameHeight in main.qml.
#   run:  python3 tools/gen-claude-sprite.py

from PIL import Image
import os

W, H, NF = 13, 12, 4
CLAY = (193, 110, 85, 255)   # terracotta body
EYE  = (26, 22, 21, 255)     # near-black eyes

def body(px):
    def s(x, y):
        if 0 <= x < W and 0 <= y < H:
            px[x, y] = CLAY
    for x in range(4, 9):                 # row 0: head top (cols 4-8)
        s(x, 0)
    for y in range(1, 9):                 # rows 1-8: main block (cols 3-9)
        for x in range(3, 10):
            s(x, y)
    for y in (4, 5):                       # arms out the sides (rows 4-5)
        for x in (1, 2, 10, 11):
            s(x, y)

def frame(idx):
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    px = img.load()
    body(px)

    # legs (cols 4-5 and 7-8), alternating length for the walk cycle
    legL, legR = {0: (3, 3), 1: (3, 2), 2: (2, 3), 3: (3, 3)}[idx]
    def leg(cols, n):
        for k in range(n):
            for x in cols:
                if 0 <= x < W and 0 <= 9 + k < H:
                    px[x, 9 + k] = CLAY
    leg((4, 5), legL)
    leg((7, 8), legR)

    # eyes, drawn over the body; shift right by 1 on the middle frames (look right)
    look = idx in (1, 2)
    lcols = (5, 6) if look else (4, 5)
    rcols = (8, 9) if look else (7, 8)
    for y in (2, 3):
        for x in lcols + rcols:
            px[x, y] = EYE
    return img

sheet = Image.new("RGBA", (W * NF, H), (0, 0, 0, 0))
for i in range(NF):
    sheet.paste(frame(i), (i * W, 0))

OUT = os.path.join(os.path.dirname(__file__), "..", "plasmoid", "contents", "icons")
sheet.save(os.path.join(OUT, "claude-walk.png"))
frame(0).save(os.path.join(OUT, "claude-still.png"))

# 12x preview for review
sheet.resize((W * NF * 12, H * 12), Image.NEAREST).save("/tmp/claw-preview.png")
print("done", sheet.size)
