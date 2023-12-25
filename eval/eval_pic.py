from PIL import Image
from PIL import ImageFile
from io import BytesIO
import numpy as np
import sys
import time
import torch
import requests
import deep_danbooru_model
import base64

ImageFile.LOAD_TRUNCATED_IMAGES = True

flag_use_cuda = False
flag_use_ocl = False

flag_eprint_time = False
flag_eprint_eof = False

def eprint_timestamp(message):
    if flag_eprint_time:
        print(f"{message:40} {time.time()}", file=sys.stderr)

eprint_timestamp("start")
if flag_use_ocl:
    # require libpt_ocl.so from https://github.com/artyom-beilis/pytorch_dlprim
    torch.ops.load_library("/usr/lib/libpt_ocl.so")
    torch.utils.rename_privateuse1_backend('ocl')

model = deep_danbooru_model.DeepDanbooruModel()
model.load_state_dict(torch.load('model-resnet_custom_v3.pt'))
eprint_timestamp("stuff loaded from disk")

model.eval()
model.float()
if flag_use_cuda:
    model.cuda().half()
elif flag_use_ocl:
    model.to("ocl:0")
eprint_timestamp("finished model setup")
tag_classes = list(map(lambda a: int(a), open("tag_class").read().strip().split('\n')))
eprint_timestamp("loaded tag list")

print("---ready---")
while True:
    try:
        link = input()
    except EOFError:
        if flag_eprint_eof:
            print("EOF", file=sys.stderr)
        break
    except:
        break
    if link == "":
        break
    if link == "exit":
        break
    if link == "tags":
        eprint_timestamp("print tag list start")
        print(len(model.tags))
        for i,(name, tag_class) in enumerate(zip(model.tags, tag_classes)):
            print(i, name, tag_class)
        eprint_timestamp("print tag list end")
        continue
    
    image = 0
    eprint_timestamp("retrieve image start")
    if link.startswith("bin "):
        image_length = int(link.split(' ')[1])
        # not implemented yet
        image = 0
    
    elif link.startswith("base64 "):
        image = base64.b64decode(link.split(' ')[1])
    
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
        # time.sleep(10)
        image = image.content
    eprint_timestamp("retrieve image end")

    eprint_timestamp("convert image start")
    try:
        image = Image.open(BytesIO(image)).convert("RGB").resize((512, 512))
    except:
        print("-3")
        continue
    
    eprint_timestamp("convert image end")
    eprint_timestamp("prepare data start")
    a = np.expand_dims(np.array(image, dtype=np.float32), 0) / 255
    x = torch.from_numpy(a)
    if flag_use_cuda:
        x = x.cuda().half()
    elif flag_use_ocl:
        x = x.to("ocl:0")
    eprint_timestamp("prepare data end")
    eprint_timestamp("inference start")
    y = model(x)[0].detach().cpu().numpy()
    eprint_timestamp("inference end")
    print(len(list(filter(lambda a: a[1]>=0.2,enumerate(y)))))
    for i, p in enumerate(y):
        if p >= 0.2:
            print(i, p)
