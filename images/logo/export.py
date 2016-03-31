import os
import errno
import subprocess


def mkdir_p(path):
    try:
        os.makedirs(path)
    except OSError as exc:  # Python >2.5
        if exc.errno == errno.EEXIST and os.path.isdir(path):
            pass
        else:
            raise

sizes = {
    "ldpi": 32,
    "mdpi": 48,
    "hdpi": 72,
    "xhdpi": 96,
    "xxhdpi": 144,
    "xxxhdpi": 192
}
for name in sizes:
    size = sizes[name]
    output_folder_base = "../../android/res"
    full_name = "drawable-" + name
    output_folder = os.path.join(output_folder_base, full_name)
    mkdir_p(output_folder)
    filename = os.path.join(output_folder, "icon.png")
    command = "inkscape --export-png " + filename + " -w " + str(size) + " neuronify_logo.svg"
    print(command)
    subprocess.call(command, shell=True)
