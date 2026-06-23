# Generates the walking-Claude pixel-art sprite for the plasmoid:
#   plasmoid/contents/icons/claude-walk.png   (4-frame horizontal sheet, 16x14/frame)
#   plasmoid/contents/icons/claude-still.png  (standing frame, for the popup)
# A hand-made approximation of Anthropic's pixel mascot (legs walk, eyes dart right).
# To use the OFFICIAL asset instead: drop a 4-frame 16x14 sheet at claude-walk.png
# (or adjust frameCount/frameWidth/frameHeight in main.qml for other dimensions).
#   run:  python3 tools/gen-claude-sprite.py

from PIL import Image

W, H, NF = 16, 14, 4
BODY=(216,118,86,255); SH=(183,92,64,255)
EYE=(250,247,240,255); PUP=(40,30,26,255); LEG=(176,88,60,255)

def frame(idx):
    img = Image.new("RGBA",(W,H),(0,0,0,0)); px=img.load()
    def s(x,y,c):
        if 0<=x<W and 0<=y<H: px[x,y]=c
    # rounded body x2..13 y1..9
    for y in range(1,10):
        for x in range(2,14):
            if (x in (2,13)) and (y in (1,9)): continue   # cut corners
            s(x,y,BODY)
    for x in range(3,13): s(x,9,SH)        # bottom shade
    # little antennae
    s(5,0,SH); s(10,0,SH)
    # eyes (whites) left x4..5 right x9..10, y3..5
    for y in range(3,6):
        for x in (4,5,9,10): s(x,y,EYE)
    look = idx in (1,2)                    # dart right on middle frames
    lx = 5 if look else 4
    rx = 10 if look else 9
    for y in (4,5): s(lx,y,PUP); s(rx,y,PUP)
    # legs x3,9 = group A ; x6,12 = group B ; alternate for walk
    aLen,bLen = [(2,1),(1,1),(1,2),(1,1)][idx]
    def leg(x,n):
        for k in range(n): s(x,10+k,LEG)
    leg(3,aLen); leg(9,aLen); leg(6,bLen); leg(12,bLen)
    return img

# sprite sheet (4 frames horizontal)
sheet = Image.new("RGBA",(W*NF,H),(0,0,0,0))
for i in range(NF):
    sheet.paste(frame(i),(i*W,0))
import os
OUT=os.path.join(os.path.dirname(__file__), "..", "plasmoid", "contents", "icons")
sheet.save(f"{OUT}/claude-walk.png")
# static standing frame for popup
still = Image.new("RGBA",(W,H),(0,0,0,0)); still.paste(frame(3),(0,0))
still.save(f"{OUT}/claude-still.png")

# big preview (10x nearest) of all frames stacked for review
prev = sheet.resize((W*NF*10, H*10), Image.NEAREST)
prev.save("/tmp/claude-walk-preview.png")
print("wrote claude-walk.png (sheet", sheet.size, ") + claude-still.png")
