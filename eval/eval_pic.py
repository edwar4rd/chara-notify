from PIL import Image
from io import BytesIO
import numpy as np
import sys
import time
import torch
import requests
import deep_danbooru_model

model = deep_danbooru_model.DeepDanbooruModel()
model.load_state_dict(torch.load('model-resnet_custom_v3.pt'))

model.eval()
model.float()
tag_classes = list(map(lambda a: int(a), open("tag_class").read().strip().split('\n')))

while True:
    link = input()
    if link == "tags":
        for i,(name, tag_class) in enumerate(zip(model.tags, tag_classes)):
            print(i, name, tag_class)
    
    else:
        start = time.time()
        sys.settrace(f := lambda a,b,c: (0 if time.time() - start <= 20 else ([] for [] in []).throw(Exception("timeout")),f)[1])
        try:
            image = requests.get(link, timeout=(3, 6))
        
        except:
            print("-1")
            sys.settrace(None)
            continue
        
        finally:
            sys.settrace(None)
        
        if not image.ok:
            print("-2")
            continue
        
        image = image.content
        try:
            image = Image.open(BytesIO(image)).convert("RGB").resize((512, 512))
        
        except:
            print("-3")
            continue
        
        a = np.expand_dims(np.array(image, dtype=np.float32), 0) / 255
        x = torch.from_numpy(a)
        y = model(x)[0].detach().cpu().numpy()
        print(len(list(filter(lambda a: a[1]>=0.2,enumerate(y)))))
        for i, p in enumerate(y):
            if p >= 0.2:
                print(i, p)
