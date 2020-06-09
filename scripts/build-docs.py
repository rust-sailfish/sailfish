#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import shutil
import subprocess


def build_docs(input_dir: str, output_dir: str):
    subprocess.call('mkdocs build', shell=True, cwd=input_dir)
    site_dir = os.path.join(input_dir, 'site')
    shutil.copytree(site_dir, output_dir)


def main() -> None:
    if os.path.exists('site'):
        os.removedirs('site')

    os.mkdir('site')
    build_docs('./docs/en', output_dir='site/en')


if __name__ == '__main__':
    main()
