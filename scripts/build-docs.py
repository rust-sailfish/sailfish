#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import shutil
import subprocess


def build_docs(input_dir: str, output_dir: str):
    subprocess.call('python3 -m pip install --upgrade pip', shell=True, cwd=input_dir)
    subprocess.call('python3 -m pip install mkdocs', shell=True, cwd=input_dir)
    subprocess.call('python3 -m mkdocs build', shell=True, cwd=input_dir)
    site_dir = os.path.join(input_dir, 'site')
    shutil.copytree(site_dir, output_dir)


def main() -> None:
    if os.path.exists('site'):  
        if os.path.isfile('site') or os.path.islink('site'):
            os.unlink('site')
        else:
            shutil.rmtree('site')

    os.mkdir('site')
    # get the path of the current directory
    docs_path = os.path.join(os.getcwd(), "docs/en")
    print(docs_path)
    build_docs(docs_path, output_dir='site/en')


if __name__ == '__main__':
    main()
