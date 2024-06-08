#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import shutil
import subprocess


def build_docs(input_dir: str, output_dir: str):
    try:
        os.system('sudo apt install python3-pip')
    except:
        print("Couldn't install python3-pip. It may already be installed.")
    subprocess.call('python3 -m pip install --upgrade pip', shell=True, cwd=input_dir)
    subprocess.call('python3 -m pip install mkdocs', shell=True, cwd=input_dir)
    subprocess.call('python3 -m pip install mkdocs-material', shell=True, cwd=input_dir)

    subprocess.call('python3 -m mkdocs build', shell=True, cwd=input_dir)
    site_dir = os.path.join(input_dir, 'site')
    #shutil.copytree(site_dir, output_dir)


def main() -> None:
    current_directory = os.getcwd()
    if current_directory.endswith("scripts") == False:
        raise SystemExit("Please ensure you are in the scripts directory.")
    # go up to the sailfish root directory
    os.chdir('..')

    # go down to the docs folder
    os.chdir('docs')

    # go down to the en folder
    os.chdir('en')

    if os.path.exists('site'):  
        if os.path.isfile('site') or os.path.islink('site'):
            os.unlink('site')
        else:
            shutil.rmtree('site')

    os.mkdir('site')
    current_directory = os.getcwd()
    print(current_directory)
    build_docs(current_directory, output_dir='site/en')


if __name__ == '__main__':
    main()
