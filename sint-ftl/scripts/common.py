import os
import sys
import subprocess
import time

# Configuration
# scripts/common.py -> ../scripts -> .. (ROOT)
ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VENV_DIR = os.path.join(ROOT_DIR, '.venv')
VENV_BIN = os.path.join(VENV_DIR, 'bin')
PYTHON_EXEC = os.path.join(VENV_BIN, 'python3')
BUILD_MARKER = os.path.join(VENV_DIR, '.build_marker')

def run_cmd(cmd, cwd=ROOT_DIR, env=None, background=False, quiet=False):
    if not quiet:
        print(f'🚀 Running: {cmd}')
    if env is None:
        env = os.environ.copy()
    
    # Ensure venv is in path
    env['PATH'] = f'{VENV_BIN}:{env["PATH"]}'
    
    if background:
        return subprocess.Popen(cmd, shell=True, cwd=cwd, env=env)
    else:
        ret = subprocess.call(cmd, shell=True, cwd=cwd, env=env)
        if ret != 0:
            print(f'❌ Command failed: {cmd}')
            sys.exit(ret)

def check_venv():
    if not os.path.exists(VENV_DIR):
        print('🔧 Creating Python virtual environment...')
        subprocess.check_call([sys.executable, '-m', 'venv', '.venv'])
        # Install deps
        venv_python = os.path.join(VENV_BIN, 'python')
        # -q for quiet
        run_cmd(f'{venv_python} -m pip install -q -r ai/requirements.txt', quiet=True)
        run_cmd(f'{venv_python} -m pip install -q maturin', quiet=True)

def get_latest_mtime(path):
    max_mtime = 0
    if os.path.isfile(path):
        return os.path.getmtime(path)
        
    for root, _, files in os.walk(path):
        for f in files:
            if f.endswith('.rs') or f.endswith('.toml') or f == 'Cargo.lock':
                p = os.path.join(root, f)
                m = os.path.getmtime(p)
                if m > max_mtime:
                    max_mtime = m
    return max_mtime

def needs_rebuild():
    if not os.path.exists(BUILD_MARKER):
        return True
    
    last_build = os.path.getmtime(BUILD_MARKER)
    
    # Check Directories
    dirs_to_check = ['core', 'solver']
    for d in dirs_to_check:
        if get_latest_mtime(os.path.join(ROOT_DIR, d)) > last_build:
            return True

    # Check Workspace files
    if get_latest_mtime(os.path.join(ROOT_DIR, 'Cargo.toml')) > last_build:
        return True
    if os.path.exists(os.path.join(ROOT_DIR, 'Cargo.lock')):
        if get_latest_mtime(os.path.join(ROOT_DIR, 'Cargo.lock')) > last_build:
            return True
            
    return False

def build_bindings(force=False):
    if not force and not needs_rebuild():
        # print('⏩ Skipping build (up to date)')
        return

    print('🔨 Building Python Bindings...')
    # Skip generic cargo build --all, let maturin handle it
    run_cmd('maturin develop --release', cwd=os.path.join(ROOT_DIR, 'core'))
    run_cmd('maturin develop --release', cwd=os.path.join(ROOT_DIR, 'solver'))
    
    # Touch marker
    with open(BUILD_MARKER, 'w') as f:
        f.write(str(time.time()))
